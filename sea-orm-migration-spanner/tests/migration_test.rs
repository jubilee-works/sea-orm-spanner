use sea_orm_migration_spanner::prelude::*;
use serial_test::serial;

struct M20220101CreateUsers;

impl MigrationName for M20220101CreateUsers {
    fn name(&self) -> &str {
        "m20220101_000001_create_users"
    }
}

#[async_trait]
impl MigrationTrait for M20220101CreateUsers {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("test_users")
                    .string("id", Some(36), true)
                    .string("name", Some(255), true)
                    .string("email", Some(255), true)
                    .primary_key(["id"]),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table_by_name("test_users").await
    }
}

struct M20220102CreatePosts;

impl MigrationName for M20220102CreatePosts {
    fn name(&self) -> &str {
        "m20220102_000001_create_posts"
    }
}

#[async_trait]
impl MigrationTrait for M20220102CreatePosts {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("test_posts")
                    .string("id", Some(36), true)
                    .string("user_id", Some(36), true)
                    .string("title", Some(255), true)
                    .string("content", None, false)
                    .primary_key(["id"]),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table_by_name("test_posts").await
    }
}

struct TestMigrator;

impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
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
    ) // projects/local-project/instances/test-instance/databases/migration_test_db
}

async fn setup_test_database() {
    use gcloud_googleapis::spanner::admin::database::v1::{CreateDatabaseRequest, DatabaseDialect};
    use gcloud_googleapis::spanner::admin::instance::v1::{CreateInstanceRequest, Instance};
    use gcloud_spanner::admin::client::Client as AdminClient;
    use gcloud_spanner::admin::AdminClientConfig;

    if std::env::var("SPANNER_EMULATOR_HOST").is_err() {
        panic!("SPANNER_EMULATOR_HOST not set");
    }

    let project_path = format!("projects/{}", PROJECT);
    let instance_path = format!("{}/instances/{}", project_path, INSTANCE);

    let admin_client = AdminClient::new(AdminClientConfig::default())
        .await
        .unwrap();
    let _ = admin_client
        .instance()
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
        )
        .await;

    let _ = admin_client
        .database()
        .create_database(
            CreateDatabaseRequest {
                parent: instance_path,
                create_statement: format!("CREATE DATABASE `{}`", DATABASE),
                extra_statements: vec![],
                encryption_config: None,
                database_dialect: DatabaseDialect::GoogleStandardSql.into(),
                proto_descriptors: vec![],
            },
            None,
        )
        .await;
}

#[tokio::test]
#[serial]
async fn test_migration_up_and_down() {
    setup_test_database().await;

    let db_path = database_path();

    TestMigrator::fresh(&db_path).await.ok();

    let result = TestMigrator::up(&db_path, None).await;
    assert!(result.is_ok(), "Migration up failed: {:?}", result.err());

    let result = TestMigrator::status(&db_path).await;
    assert!(result.is_ok(), "Status check failed: {:?}", result.err());

    let result = TestMigrator::down(&db_path, Some(1)).await;
    assert!(result.is_ok(), "Migration down failed: {:?}", result.err());

    let result = TestMigrator::reset(&db_path).await;
    assert!(result.is_ok(), "Reset failed: {:?}", result.err());
}

#[tokio::test]
#[serial]
async fn test_migration_fresh() {
    setup_test_database().await;

    let db_path = database_path();

    let result = TestMigrator::fresh(&db_path).await;
    assert!(result.is_ok(), "Fresh failed: {:?}", result.err());

    let result = TestMigrator::reset(&db_path).await;
    assert!(result.is_ok(), "Reset failed: {:?}", result.err());
}
