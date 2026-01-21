use sea_orm::DbErr;
use sea_orm_migration_spanner::{
    SpannerMigrationTrait, SpannerMigratorTrait, SpannerSchemaManager,
};

struct M20220101CreateUsers;

#[async_trait::async_trait]
impl SpannerMigrationTrait for M20220101CreateUsers {
    fn name(&self) -> &str {
        "m20220101_000001_create_users"
    }

    async fn up(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                "CREATE TABLE test_users (
                    id STRING(36) NOT NULL,
                    name STRING(255) NOT NULL,
                    email STRING(255) NOT NULL,
                ) PRIMARY KEY (id)",
            )
            .await
    }

    async fn down(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager.drop_table("test_users").await
    }
}

struct M20220102CreatePosts;

#[async_trait::async_trait]
impl SpannerMigrationTrait for M20220102CreatePosts {
    fn name(&self) -> &str {
        "m20220102_000001_create_posts"
    }

    async fn up(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                "CREATE TABLE test_posts (
                    id STRING(36) NOT NULL,
                    user_id STRING(36) NOT NULL,
                    title STRING(255) NOT NULL,
                    content STRING(MAX),
                ) PRIMARY KEY (id)",
            )
            .await
    }

    async fn down(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager.drop_table("test_posts").await
    }
}

struct TestMigrator;

impl SpannerMigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn SpannerMigrationTrait>> {
        vec![
            Box::new(M20220101CreateUsers),
            Box::new(M20220102CreatePosts),
        ]
    }
}

const PROJECT: &str = "local-project";
const INSTANCE: &str = "test-instance";
const DATABASE: &str = "migration_test_db";

fn database_path() -> String {
    format!(
        "projects/{}/instances/{}/databases/{}",
        PROJECT, INSTANCE, DATABASE
    )
}

async fn setup_test_database() {
    use google_cloud_googleapis::spanner::admin::database::v1::{
        CreateDatabaseRequest, DatabaseDialect,
    };
    use google_cloud_googleapis::spanner::admin::instance::v1::{CreateInstanceRequest, Instance};
    use google_cloud_spanner::admin::database::database_admin_client::DatabaseAdminClient;
    use google_cloud_spanner::admin::instance::instance_admin_client::InstanceAdminClient;

    if std::env::var("SPANNER_EMULATOR_HOST").is_err() {
        panic!("SPANNER_EMULATOR_HOST not set");
    }

    let project_path = format!("projects/{}", PROJECT);
    let instance_path = format!("{}/instances/{}", project_path, INSTANCE);

    let mut instance_client = InstanceAdminClient::default().await.unwrap();
    let _ = instance_client
        .create_instance(
            CreateInstanceRequest {
                parent: project_path,
                instance_id: INSTANCE.to_string(),
                instance: Some(Instance {
                    name: instance_path.clone(),
                    config: "".to_string(),
                    display_name: "Test Instance".to_string(),
                    ..Default::default()
                }),
            },
            None,
            None,
        )
        .await;

    let db_client = DatabaseAdminClient::default().await.unwrap();
    let _ = db_client
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
}

#[tokio::test]
async fn test_migration_up_and_down() {
    setup_test_database().await;

    let db_path = database_path();

    TestMigrator::fresh(&db_path).await.ok();

    let result = TestMigrator::up(&db_path, None).await;
    assert!(result.is_ok(), "Migration up failed: {:?}", result.err());

    let result = TestMigrator::status(&db_path).await;
    assert!(result.is_ok(), "Status check failed: {:?}", result.err());

    let result = TestMigrator::down(&db_path, Some(1)).await;
    assert!(
        result.is_ok(),
        "Migration down failed: {:?}",
        result.err()
    );

    let result = TestMigrator::reset(&db_path).await;
    assert!(result.is_ok(), "Reset failed: {:?}", result.err());
}

#[tokio::test]
async fn test_migration_fresh() {
    setup_test_database().await;

    let db_path = database_path();

    let result = TestMigrator::fresh(&db_path).await;
    assert!(result.is_ok(), "Fresh failed: {:?}", result.err());

    let result = TestMigrator::reset(&db_path).await;
    assert!(result.is_ok(), "Reset failed: {:?}", result.err());
}
