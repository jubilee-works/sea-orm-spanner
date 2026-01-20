use sea_orm::DbBackend;
use sea_orm_spanner::SpannerDatabase;

mod entities;

const PROJECT: &str = "local-project";
const INSTANCE: &str = "test-instance";
const DATABASE: &str = "test-database";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sea-ORM Spanner Example");
    
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    
    setup_emulator_database().await?;
    
    let database_path = format!("projects/{}/instances/{}/databases/{}", PROJECT, INSTANCE, DATABASE);
    let db = SpannerDatabase::connect_with_emulator(&database_path).await?;

    println!("Connected to Spanner emulator");
    
    let user_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    
    let insert_sql = format!(
        "INSERT INTO users (id, name, email, created_at) VALUES ('{}', 'Test User', 'test@example.com', TIMESTAMP '{}')",
        user_id,
        now.format("%Y-%m-%dT%H:%M:%S%.6fZ")
    );
    
    println!("Executing: {}", insert_sql);
    let stmt = sea_orm::Statement::from_string(DbBackend::Postgres, insert_sql);
    let rows_affected = db.execute(stmt).await?;
    println!("Inserted {} row(s)", rows_affected);
    
    let select_sql = "SELECT id, name, email FROM users LIMIT 10";
    println!("Executing: {}", select_sql);
    let stmt = sea_orm::Statement::from_string(DbBackend::Postgres, select_sql.to_string());
    let results = db.query_all(stmt).await?;
    
    println!("Found {} user(s):", results.len());
    for row in results {
        let id: String = row.try_get_by_name("id").map_err(|e| format!("{:?}", e))?;
        let name: String = row.try_get_by_name("name").map_err(|e| format!("{:?}", e))?;
        let email: String = row.try_get_by_name("email").map_err(|e| format!("{:?}", e))?;
        println!("  - {} | {} | {}", id, name, email);
    }

    db.close().await;
    
    Ok(())
}

async fn setup_emulator_database() -> Result<(), Box<dyn std::error::Error>> {
    use google_cloud_spanner::admin::instance::instance_admin_client::InstanceAdminClient;
    use google_cloud_spanner::admin::database::database_admin_client::DatabaseAdminClient;
    use google_cloud_googleapis::spanner::admin::instance::v1::{CreateInstanceRequest, Instance};
    use google_cloud_googleapis::spanner::admin::database::v1::{CreateDatabaseRequest, DatabaseDialect};
    
    let project_path = format!("projects/{}", PROJECT);
    let instance_path = format!("{}/instances/{}", project_path, INSTANCE);
    
    let mut instance_client = InstanceAdminClient::default().await?;
    let create_result = instance_client.create_instance(
        CreateInstanceRequest {
            parent: project_path.clone(),
            instance_id: INSTANCE.to_string(),
            instance: Some(Instance {
                name: instance_path.clone(),
                config: "".to_string(),
                display_name: "Test Instance".to_string(),
                node_count: 0,
                processing_units: 0,
                state: 0,
                labels: Default::default(),
                endpoint_uris: vec![],
                create_time: None,
                update_time: None,
            }),
        },
        None,
        None,
    ).await;
    
    match create_result {
        Ok(mut op) => {
            let _ = op.wait(None, None).await;
            println!("Instance created");
        }
        Err(e) => {
            if e.to_string().contains("ALREADY_EXISTS") {
                println!("Instance already exists");
            } else {
                println!("Instance creation error (may already exist): {}", e);
            }
        }
    }
    
    let db_client = DatabaseAdminClient::default().await?;
    let db_result = db_client.create_database(
        CreateDatabaseRequest {
            parent: instance_path.clone(),
            create_statement: format!("CREATE DATABASE `{}`", DATABASE),
            extra_statements: vec![
                "CREATE TABLE users (
                    id STRING(36) NOT NULL,
                    name STRING(255) NOT NULL,
                    email STRING(255) NOT NULL,
                    created_at TIMESTAMP NOT NULL,
                ) PRIMARY KEY (id)".to_string(),
            ],
            encryption_config: None,
            database_dialect: DatabaseDialect::GoogleStandardSql.into(),
        },
        None,
        None,
    ).await;
    
    match db_result {
        Ok(mut op) => {
            let _ = op.wait(None, None).await;
            println!("Database created");
        }
        Err(e) => {
            if e.to_string().contains("ALREADY_EXISTS") {
                println!("Database already exists");
            } else {
                println!("Database creation error (may already exist): {}", e);
            }
        }
    }
    
    Ok(())
}
