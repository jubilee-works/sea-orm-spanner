use clap::{Parser, Subcommand};
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
    #[command(about = "Initialize migration directory")]
    Init {
        #[arg(short, long, default_value = "./migration", help = "Migration directory path")]
        dir: String,
    },

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

fn require_database(database: Option<String>) -> String {
    database.expect("DATABASE_PATH is required. Use -d or set DATABASE_PATH env var.")
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { dir } => {
            if let Err(e) = sea_orm_migration_spanner::run_migrate_init(&dir) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Generate { name } => {
            if let Err(e) = sea_orm_migration_spanner::run_migrate_generate("./src", &name) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Up { num } => {
            let db = require_database(cli.database);
            if let Err(e) = Migrator::up(&db, num).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Down { num } => {
            let db = require_database(cli.database);
            if let Err(e) = Migrator::down(&db, Some(num)).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Status => {
            let db = require_database(cli.database);
            if let Err(e) = Migrator::status(&db).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Fresh => {
            let db = require_database(cli.database);
            if let Err(e) = Migrator::fresh(&db).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Reset => {
            let db = require_database(cli.database);
            if let Err(e) = Migrator::reset(&db).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
