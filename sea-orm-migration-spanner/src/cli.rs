use crate::MigratorTrait;
use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Parser)]
#[command(name = "migration")]
#[command(about = "Spanner database migration tool")]
struct Cli {
    #[arg(
        short = 'u',
        long = "database-url",
        env = "DATABASE_URL",
        help = "Spanner database path in format: projects/{project}/instances/{instance}/databases/{database}"
    )]
    database_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long, default_value = "./migration")]
        dir: String,
    },
    Generate {
        name: String,
    },
    Up {
        #[arg(short, long)]
        num: Option<u32>,
    },
    Down {
        #[arg(short, long, default_value = "1")]
        num: u32,
    },
    Status,
    Fresh,
    Reset,
}

pub async fn run_cli<M: MigratorTrait>(_migrator: M) {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let result: Result<(), Box<dyn std::error::Error>> = match cli.command {
        Commands::Init { dir } => run_migrate_init(&dir),
        Commands::Generate { name } => run_migrate_generate("./src", &name),
        Commands::Up { num } => {
            let db = require_database_url(cli.database_url);
            M::up(&db, num).await.map_err(|e| e.into())
        }
        Commands::Down { num } => {
            let db = require_database_url(cli.database_url);
            M::down(&db, Some(num)).await.map_err(|e| e.into())
        }
        Commands::Status => {
            let db = require_database_url(cli.database_url);
            M::status(&db).await.map_err(|e| e.into())
        }
        Commands::Fresh => {
            let db = require_database_url(cli.database_url);
            M::fresh(&db).await.map_err(|e| e.into())
        }
        Commands::Reset => {
            let db = require_database_url(cli.database_url);
            M::reset(&db).await.map_err(|e| e.into())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn require_database_url(database_url: Option<String>) -> String {
    database_url
        .expect("DATABASE_URL is required. Use -u/--database-url or set DATABASE_URL env var.")
}

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

use sea_orm_migration_spanner::prelude::*;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
        ]
    }
}
"#;

    let main_rs = r#"use sea_orm_migration_spanner::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
"#;

    let migration_template = r#"use sea_orm_migration_spanner::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table_raw(
                "CREATE TABLE example_table (
                    id STRING(36) NOT NULL,
                    name STRING(255) NOT NULL,
                ) PRIMARY KEY (id)",
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table_by_name("example_table").await
    }
}
"#;

    let readme = r#"# Migration

Spanner database migrations using sea-orm-migration-spanner.

## Usage

```bash
# Set database URL (Spanner uses path format: projects/{project}/instances/{instance}/databases/{database})
export DATABASE_URL="projects/my-project/instances/my-instance/databases/my-db"

# For local development with emulator
export SPANNER_EMULATOR_HOST=localhost:9010

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
        r#"use sea_orm_migration_spanner::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        todo!("Implement migration up")
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
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
