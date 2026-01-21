use google_cloud_googleapis::spanner::admin::database::v1::{
    CreateDatabaseRequest, DatabaseDialect, UpdateDatabaseDdlRequest,
};
use google_cloud_googleapis::spanner::admin::instance::v1::{CreateInstanceRequest, Instance};
use google_cloud_spanner::admin::database::database_admin_client::DatabaseAdminClient;
use google_cloud_spanner::admin::instance::instance_admin_client::InstanceAdminClient;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use sea_orm_spanner::SpannerDatabase;
use std::sync::atomic::{AtomicBool, Ordering};

pub const PROJECT: &str = "local-project";
pub const INSTANCE: &str = "test-instance";
pub const DATABASE: &str = "test_db";

static INITIALIZED: AtomicBool = AtomicBool::new(false);

const ALL_DDL: &[&str] = &[
    "CREATE TABLE users (
        id STRING(36) NOT NULL,
        name STRING(255) NOT NULL,
        email STRING(255) NOT NULL,
        age INT64,
        active BOOL NOT NULL,
        created_at TIMESTAMP NOT NULL,
    ) PRIMARY KEY (id)",
    "CREATE TABLE posts (
        id STRING(36) NOT NULL,
        user_id STRING(36) NOT NULL,
        title STRING(255) NOT NULL,
        content STRING(MAX) NOT NULL,
        published BOOL NOT NULL,
        created_at TIMESTAMP NOT NULL,
    ) PRIMARY KEY (id)",
    "CREATE TABLE categories (
        id STRING(36) NOT NULL,
        name STRING(255) NOT NULL,
        description STRING(MAX),
    ) PRIMARY KEY (id)",
    "CREATE TABLE products (
        id STRING(36) NOT NULL,
        category_id STRING(36) NOT NULL,
        name STRING(255) NOT NULL,
        price FLOAT64 NOT NULL,
        quantity INT64 NOT NULL,
        active BOOL NOT NULL,
    ) PRIMARY KEY (id)",
];

const ALL_TABLES: &[&str] = &["users", "posts", "categories", "products"];

pub async fn setup_test_database() -> DatabaseConnection {
    if std::env::var("SPANNER_EMULATOR_HOST").is_err() {
        panic!("SPANNER_EMULATOR_HOST not set. Run: export SPANNER_EMULATOR_HOST=localhost:9010");
    }

    if !INITIALIZED.load(Ordering::SeqCst) {
        setup_instance().await.ok();
        ensure_database_exists().await.expect("Failed to create database");
        ensure_tables_exist().await;
        INITIALIZED.store(true, Ordering::SeqCst);
    }

    let database_path = format!(
        "projects/{}/instances/{}/databases/{}",
        PROJECT, INSTANCE, DATABASE
    );
    let db = SpannerDatabase::connect(&database_path)
        .await
        .expect("Failed to connect to database");

    clear_tables(&db).await;

    db
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

async fn ensure_database_exists() -> Result<(), Box<dyn std::error::Error>> {
    let instance_path = format!("projects/{}/instances/{}", PROJECT, INSTANCE);

    let db_client = DatabaseAdminClient::default().await?;
    let result = db_client
        .create_database(
            CreateDatabaseRequest {
                parent: instance_path,
                create_statement: format!("CREATE DATABASE `{}`", DATABASE),
                extra_statements: vec![],
                encryption_config: None,
                database_dialect: DatabaseDialect::GoogleStandardSql.into(),
            },
            None,
            None,
        )
        .await;

    match result {
        Ok(mut op) => {
            let _ = op.wait(None, None).await;
        }
        Err(e) => {
            let err_str = e.to_string();
            if !err_str.contains("AlreadyExists") && !err_str.contains("already exists") {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

async fn ensure_tables_exist() {
    let instance_path = format!("projects/{}/instances/{}", PROJECT, INSTANCE);
    let database_path = format!("{}/databases/{}", instance_path, DATABASE);

    let db_client = DatabaseAdminClient::default()
        .await
        .expect("Failed to create admin client");

    // Try to create each table individually, ignoring "already exists" errors
    for ddl in ALL_DDL {
        let result = db_client
            .update_database_ddl(
                UpdateDatabaseDdlRequest {
                    database: database_path.clone(),
                    statements: vec![ddl.to_string()],
                    operation_id: "".to_string(),
                },
                None,
                None,
            )
            .await;

        if let Ok(mut op) = result {
            let _ = op.wait(None, None).await;
        }
        // Ignore errors (table may already exist)
    }
}

async fn clear_tables(db: &DatabaseConnection) {
    for table in ALL_TABLES {
        let sql = format!("DELETE FROM {} WHERE true", table);
        let _ = db
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::MySql,
                sql,
            ))
            .await;
    }
}
