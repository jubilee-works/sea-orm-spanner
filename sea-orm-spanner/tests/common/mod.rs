use google_cloud_googleapis::spanner::admin::database::v1::{
    CreateDatabaseRequest, DatabaseDialect,
};
use google_cloud_googleapis::spanner::admin::instance::v1::{CreateInstanceRequest, Instance};
use google_cloud_spanner::admin::database::database_admin_client::DatabaseAdminClient;
use google_cloud_spanner::admin::instance::instance_admin_client::InstanceAdminClient;
use sea_orm::DatabaseConnection;
use sea_orm_spanner::SpannerDatabase;
use std::sync::atomic::{AtomicBool, Ordering};

pub const PROJECT: &str = "local-project";
pub const INSTANCE: &str = "test-instance";

static INSTANCE_CREATED: AtomicBool = AtomicBool::new(false);

pub async fn setup_test_database(
    database_name: &str,
    ddl_statements: Vec<&str>,
) -> DatabaseConnection {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");

    if !INSTANCE_CREATED.load(Ordering::Relaxed) {
        setup_instance().await.ok();
        INSTANCE_CREATED.store(true, Ordering::Relaxed);
    }

    setup_database(database_name, ddl_statements)
        .await
        .expect("Failed to create database");

    let database_path = format!(
        "projects/{}/instances/{}/databases/{}",
        PROJECT, INSTANCE, database_name
    );
    SpannerDatabase::connect(&database_path)
        .await
        .expect("Failed to connect to database")
}

async fn setup_instance() -> Result<(), Box<dyn std::error::Error>> {
    let project_path = format!("projects/{}", PROJECT);
    let instance_path = format!("{}/instances/{}", project_path, INSTANCE);

    let mut instance_client = InstanceAdminClient::default().await?;
    let result = instance_client
        .create_instance(
            CreateInstanceRequest {
                parent: project_path,
                instance_id: INSTANCE.to_string(),
                instance: Some(Instance {
                    name: instance_path,
                    config: "".to_string(),
                    display_name: "Test Instance".to_string(),
                    ..Default::default()
                }),
            },
            None,
            None,
        )
        .await;

    if let Ok(mut op) = result {
        let _ = op.wait(None, None).await;
    }

    Ok(())
}

async fn setup_database(
    database_name: &str,
    ddl_statements: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let instance_path = format!("projects/{}/instances/{}", PROJECT, INSTANCE);

    let db_client = DatabaseAdminClient::default().await?;
    let result = db_client
        .create_database(
            CreateDatabaseRequest {
                parent: instance_path,
                create_statement: format!("CREATE DATABASE `{}`", database_name),
                extra_statements: ddl_statements.into_iter().map(|s| s.to_string()).collect(),
                encryption_config: None,
                database_dialect: DatabaseDialect::GoogleStandardSql.into(),
            },
            None,
            None,
        )
        .await;

    match result {
        Ok(mut op) => {
            op.wait(None, None).await?;
        }
        Err(e) => {
            if !e.to_string().contains("ALREADY_EXISTS") {
                return Err(e.into());
            }
        }
    }

    Ok(())
}
