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
    database: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

    let result = match cli.command {
        Commands::Up { num } => Migrator::up(&cli.database, num).await,
        Commands::Down { num } => Migrator::down(&cli.database, Some(num)).await,
        Commands::Status => Migrator::status(&cli.database).await,
        Commands::Fresh => Migrator::fresh(&cli.database).await,
        Commands::Reset => Migrator::reset(&cli.database).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
