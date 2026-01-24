use gcloud_googleapis::spanner::admin::database::v1::{
    CreateDatabaseRequest, DatabaseDialect, UpdateDatabaseDdlRequest,
};
use gcloud_googleapis::spanner::admin::instance::v1::{CreateInstanceRequest, Instance};
use gcloud_spanner::admin::client::Client as AdminClient;
use gcloud_spanner::admin::AdminClientConfig;
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
    "CREATE TABLE all_types (
        id STRING(36) NOT NULL,
        string_val STRING(MAX) NOT NULL,
        string_nullable STRING(MAX),
        int64_val INT64 NOT NULL,
        int64_nullable INT64,
        int32_val INT64 NOT NULL,
        int32_nullable INT64,
        float64_val FLOAT64 NOT NULL,
        float64_nullable FLOAT64,
        float32_val FLOAT64 NOT NULL,
        float32_nullable FLOAT64,
        bool_val BOOL NOT NULL,
        bool_nullable BOOL,
        bytes_val BYTES(MAX) NOT NULL,
        bytes_nullable BYTES(MAX),
        timestamp_val TIMESTAMP NOT NULL,
        timestamp_nullable TIMESTAMP,
        json_val JSON NOT NULL,
        json_nullable JSON,
    ) PRIMARY KEY (id)",
    "CREATE TABLE array_types (
        id STRING(36) NOT NULL,
        int64_array ARRAY<INT64> NOT NULL,
        int64_array_nullable ARRAY<INT64>,
        float64_array ARRAY<FLOAT64> NOT NULL,
        float64_array_nullable ARRAY<FLOAT64>,
        string_array ARRAY<STRING(MAX)> NOT NULL,
        string_array_nullable ARRAY<STRING(MAX)>,
        bool_array ARRAY<BOOL> NOT NULL,
        bool_array_nullable ARRAY<BOOL>,
    ) PRIMARY KEY (id)",
    "CREATE TABLE numeric_types (
        id STRING(36) NOT NULL,
        numeric_val NUMERIC NOT NULL,
        numeric_nullable NUMERIC,
    ) PRIMARY KEY (id)",
    "CREATE TABLE uuid_types (
        id STRING(36) NOT NULL,
        uuid_val UUID NOT NULL,
        uuid_nullable UUID,
    ) PRIMARY KEY (id)",
];

const ALL_TABLES: &[&str] = &["users", "posts", "categories", "products", "all_types", "array_types", "numeric_types", "uuid_types"];

pub async fn setup_test_database() -> DatabaseConnection {
    if std::env::var("SPANNER_EMULATOR_HOST").is_err() {
        panic!("SPANNER_EMULATOR_HOST not set. Run: export SPANNER_EMULATOR_HOST=localhost:9010");
    }

    if !INITIALIZED.load(Ordering::SeqCst) {
        setup_instance().await.ok();
        ensure_database_exists()
            .await
            .expect("Failed to create database");
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

    let admin_client = AdminClient::new(AdminClientConfig::default()).await?;
    let result = admin_client
        .instance()
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
        )
        .await;

    if let Ok(mut op) = result {
        let _ = op.wait(None).await;
    }

    Ok(())
}

async fn ensure_database_exists() -> Result<(), Box<dyn std::error::Error>> {
    let instance_path = format!("projects/{}/instances/{}", PROJECT, INSTANCE);

    let admin_client = AdminClient::new(AdminClientConfig::default()).await?;
    let result = admin_client
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

    match result {
        Ok(mut op) => {
            let _ = op.wait(None).await;
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

    let admin_client = AdminClient::new(AdminClientConfig::default())
        .await
        .expect("Failed to create admin client");

    for ddl in ALL_DDL {
        let result = admin_client
            .database()
            .update_database_ddl(
                UpdateDatabaseDdlRequest {
                    database: database_path.clone(),
                    statements: vec![ddl.to_string()],
                    operation_id: "".to_string(),
                    proto_descriptors: Default::default(),
                    throughput_mode: false,
                },
                None,
            )
            .await;

        if let Ok(mut op) = result {
            let _ = op.wait(None).await;
        }
    }
}

async fn clear_tables(db: &DatabaseConnection) {
    for table in ALL_TABLES {
        let sql = format!("DELETE FROM {} WHERE true", table);
        let _ = db
            .execute(Statement::from_string(sea_orm::DatabaseBackend::MySql, sql))
            .await;
    }
}
