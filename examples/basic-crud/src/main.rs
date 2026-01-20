use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, Set};
use sea_orm_spanner::SpannerDatabase;

mod entities;
use entities::user;

const PROJECT: &str = "local-project";
const INSTANCE: &str = "test-instance";
const DATABASE: &str = "example_db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sea-ORM Spanner ActiveRecord Example");
    println!("=====================================\n");

    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");

    setup_emulator_database().await?;

    let database_path = format!(
        "projects/{}/instances/{}/databases/{}",
        PROJECT, INSTANCE, DATABASE
    );
    let db = SpannerDatabase::connect(&database_path).await?;

    println!("✓ Connected to Spanner emulator\n");

    println!("--- INSERT ---");
    let user_id = uuid::Uuid::new_v4().to_string();
    let new_user = user::ActiveModel {
        id: Set(user_id.clone()),
        name: Set("Alice".to_string()),
        email: Set("alice@example.com".to_string()),
        created_at: Set(Utc::now()),
    };

    let inserted = new_user.insert(&db).await?;
    println!(
        "Inserted user: {} ({}) - {}",
        inserted.name, inserted.id, inserted.email
    );

    println!("\n--- SELECT by ID ---");
    let found = user::Entity::find_by_id(&user_id).one(&db).await?;
    if let Some(u) = found {
        println!("Found: {} - {}", u.name, u.email);
    }

    println!("\n--- INSERT more users ---");
    for (name, email) in [("Bob", "bob@example.com"), ("Charlie", "charlie@example.com")] {
        let u = user::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            name: Set(name.to_string()),
            email: Set(email.to_string()),
            created_at: Set(Utc::now()),
        };
        u.insert(&db).await?;
        println!("  Inserted: {}", name);
    }

    println!("\n--- SELECT all ---");
    let all_users = user::Entity::find().all(&db).await?;
    println!("Total users: {}", all_users.len());
    for u in &all_users {
        println!("  - {} ({})", u.name, u.email);
    }

    println!("\n--- UPDATE ---");
    let user_to_update = user::Entity::find_by_id(&user_id).one(&db).await?.unwrap();
    let mut active: user::ActiveModel = user_to_update.into();
    active.name = Set("Alice Updated".to_string());
    let updated = active.update(&db).await?;
    println!("Updated user: {} -> {}", user_id, updated.name);

    println!("\n--- SELECT with filter ---");
    let alice = user::Entity::find()
        .filter(user::Column::Name.contains("Alice"))
        .all(&db)
        .await?;
    println!("Users containing 'Alice': {}", alice.len());
    for u in &alice {
        println!("  - {}", u.name);
    }

    println!("\n--- COUNT ---");
    let count = user::Entity::find().count(&db).await?;
    println!("Total user count: {}", count);

    println!("\n--- DELETE ---");
    let user_to_delete = user::Entity::find_by_id(&user_id).one(&db).await?.unwrap();
    let delete_result = user_to_delete.delete(&db).await?;
    println!("Deleted {} row(s)", delete_result.rows_affected);

    let final_count = user::Entity::find().count(&db).await?;
    println!("Remaining users: {}", final_count);

    println!("\n✓ Example completed successfully!");

    Ok(())
}

async fn setup_emulator_database() -> Result<(), Box<dyn std::error::Error>> {
    use google_cloud_googleapis::spanner::admin::database::v1::{
        CreateDatabaseRequest, DatabaseDialect,
    };
    use google_cloud_googleapis::spanner::admin::instance::v1::{CreateInstanceRequest, Instance};
    use google_cloud_spanner::admin::database::database_admin_client::DatabaseAdminClient;
    use google_cloud_spanner::admin::instance::instance_admin_client::InstanceAdminClient;

    let project_path = format!("projects/{}", PROJECT);
    let instance_path = format!("{}/instances/{}", project_path, INSTANCE);

    let mut instance_client = InstanceAdminClient::default().await?;
    let create_result = instance_client
        .create_instance(
            CreateInstanceRequest {
                parent: project_path.clone(),
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

    match create_result {
        Ok(mut op) => {
            let _ = op.wait(None, None).await;
        }
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if !msg.contains("already exists") && !msg.contains("already_exists") {
                return Err(e.into());
            }
        }
    }

    let db_client = DatabaseAdminClient::default().await?;
    let db_result = db_client
        .create_database(
            CreateDatabaseRequest {
                parent: instance_path.clone(),
                create_statement: format!("CREATE DATABASE `{}`", DATABASE),
                extra_statements: vec![
                    "CREATE TABLE users (
                    id STRING(36) NOT NULL,
                    name STRING(255) NOT NULL,
                    email STRING(255) NOT NULL,
                    created_at TIMESTAMP NOT NULL,
                ) PRIMARY KEY (id)"
                        .to_string(),
                ],
                encryption_config: None,
                database_dialect: DatabaseDialect::GoogleStandardSql.into(),
            },
            None,
            None,
        )
        .await;

    match db_result {
        Ok(mut op) => {
            let _ = op.wait(None, None).await;
        }
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if !msg.contains("already exists") && !msg.contains("already_exists") {
                return Err(e.into());
            }
        }
    }

    Ok(())
}
