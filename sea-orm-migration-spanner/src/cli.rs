use std::fs;
use std::io::Write;
use std::path::Path;

pub fn run_migrate_init(migration_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let migration_dir = if migration_dir.ends_with('/') {
        migration_dir.to_string()
    } else {
        format!("{}/", migration_dir)
    };

    println!("Initializing migration directory...");

    let cargo_toml = format!(
        r#"[package]
name = "migration"
version = "0.1.0"
edition = "2021"

[dependencies]
sea-orm-migration-spanner = {{ path = "path/to/sea-orm-migration-spanner" }}
sea-orm = {{ version = "1.1", features = ["runtime-tokio-native-tls"] }}
tokio = {{ version = "1", features = ["full"] }}
async-trait = "0.1"
clap = {{ version = "4", features = ["derive", "env"] }}
dotenvy = "0.15"
"#
    );

    let lib_rs = r#"mod m20220101_000001_create_table;

use sea_orm_migration_spanner::{SpannerMigrationTrait, SpannerMigratorTrait};

pub struct Migrator;

impl SpannerMigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn SpannerMigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
        ]
    }
}
"#;

    let main_rs = r#"use clap::{Parser, Subcommand};
use migration::Migrator;
use sea_orm_migration_spanner::SpannerMigratorTrait;

#[derive(Parser)]
#[command(name = "migration")]
#[command(about = "Spanner database migration tool")]
struct Cli {
    #[arg(
        short,
        long,
        env = "DATABASE_PATH",
        help = "Spanner database path (projects/{project}/instances/{instance}/databases/{database})"
    )]
    database: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate a new migration file")]
    Generate {
        #[arg(help = "Name of the migration")]
        name: String,
    },

    #[command(about = "Apply pending migrations")]
    Up {
        #[arg(short, long, help = "Number of migrations to apply")]
        num: Option<u32>,
    },

    #[command(about = "Rollback applied migrations")]
    Down {
        #[arg(short, long, default_value = "1", help = "Number of migrations to rollback")]
        num: u32,
    },

    #[command(about = "Check the status of all migrations")]
    Status,

    #[command(about = "Drop all tables and reapply all migrations")]
    Fresh,

    #[command(about = "Rollback all applied migrations")]
    Reset,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { name } => {
            if let Err(e) = sea_orm_migration_spanner::run_migrate_generate("./src", &name) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        cmd => {
            let database = cli.database.expect("DATABASE_PATH is required for this command");
            let result = match cmd {
                Commands::Up { num } => Migrator::up(&database, num).await,
                Commands::Down { num } => Migrator::down(&database, Some(num)).await,
                Commands::Status => Migrator::status(&database).await,
                Commands::Fresh => Migrator::fresh(&database).await,
                Commands::Reset => Migrator::reset(&database).await,
                Commands::Generate { .. } => unreachable!(),
            };

            if let Err(e) = result {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
"#;

    let migration_template = r#"use sea_orm::DbErr;
use sea_orm_migration_spanner::{SpannerMigrationTrait, SpannerSchemaManager};

pub struct Migration;

#[async_trait::async_trait]
impl SpannerMigrationTrait for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }

    async fn up(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                "CREATE TABLE example_table (
                    id STRING(36) NOT NULL,
                    name STRING(255) NOT NULL,
                ) PRIMARY KEY (id)",
            )
            .await
    }

    async fn down(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager.drop_table("example_table").await
    }
}
"#;

    let readme = r#"# Migration

Spanner database migrations using sea-orm-migration-spanner.

## Usage

```bash
# Set database path
export DATABASE_PATH="projects/{project}/instances/{instance}/databases/{database}"

# Check migration status
cargo run -- status

# Apply all pending migrations
cargo run -- up

# Apply N migrations
cargo run -- up -n 1

# Rollback last migration
cargo run -- down -n 1

# Generate new migration
cargo run -- generate create_users_table

# Reset (rollback all)
cargo run -- reset

# Fresh (reset + up)
cargo run -- fresh
```
"#;

    write_file(&migration_dir, "Cargo.toml", &cargo_toml)?;
    write_file(&migration_dir, "src/lib.rs", lib_rs)?;
    write_file(&migration_dir, "src/main.rs", main_rs)?;
    write_file(
        &migration_dir,
        "src/m20220101_000001_create_table.rs",
        migration_template,
    )?;
    write_file(&migration_dir, "README.md", readme)?;

    println!("Done!");
    println!("");
    println!("Next steps:");
    println!("  1. Update Cargo.toml with correct path to sea-orm-migration-spanner");
    println!("  2. Edit src/m20220101_000001_create_table.rs with your schema");
    println!("  3. Run: cargo run -- -d 'projects/.../databases/...' up");

    Ok(())
}

pub fn run_migrate_generate(
    migration_dir: &str,
    migration_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;

    if migration_name.contains('-') {
        return Err("Migration name cannot contain hyphens. Use underscores instead.".into());
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let migration_name = migration_name.trim().replace(' ', "_");
    let file_name = format!("m{}_{}", timestamp, migration_name);

    let content = format!(
        r#"use sea_orm::DbErr;
use sea_orm_migration_spanner::{{SpannerMigrationTrait, SpannerSchemaManager}};

pub struct Migration;

#[async_trait::async_trait]
impl SpannerMigrationTrait for Migration {{
    fn name(&self) -> &str {{
        "{file_name}"
    }}

    async fn up(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {{
        todo!("Implement migration up")
    }}

    async fn down(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {{
        todo!("Implement migration down")
    }}
}}
"#
    );

    let file_path = Path::new(migration_dir).join(format!("{}.rs", file_name));
    println!("Creating migration file: {}", file_path.display());

    let mut file = fs::File::create(&file_path)?;
    file.write_all(content.as_bytes())?;

    println!("");
    println!("Don't forget to add the migration to lib.rs:");
    println!("  mod {};", file_name);
    println!("  // and add to migrations() vec:");
    println!("  Box::new({}::Migration),", file_name);

    Ok(())
}

fn write_file(
    base_dir: &str,
    filename: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let filepath = format!("{}{}", base_dir, filename);
    println!("Creating file: {}", filepath);

    let path = Path::new(&filepath);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
