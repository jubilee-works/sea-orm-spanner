#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "sea-orm-migration-spanner")]
#[command(about = "Spanner migration CLI for Sea-ORM")]
struct Cli {
    #[arg(
        short = 'd',
        long,
        env = "DATABASE_PATH",
        help = "Spanner database path (projects/{project}/instances/{instance}/databases/{database})"
    )]
    database: String,

    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    #[command(about = "Check the status of all migrations")]
    Status,

    #[command(about = "Apply pending migrations")]
    Up {
        #[arg(short, long, help = "Number of migrations to apply")]
        num: Option<u32>,
    },

    #[command(about = "Rollback applied migrations")]
    Down {
        #[arg(
            short,
            long,
            default_value = "1",
            help = "Number of migrations to rollback"
        )]
        num: u32,
    },

    #[command(about = "Drop all tables and reapply all migrations")]
    Fresh,

    #[command(about = "Rollback all applied migrations")]
    Reset,
}

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    println!("This CLI requires a Migrator implementation.");
    println!("Create a migration crate with SpannerMigratorTrait and run it directly.");
    println!("");
    println!("Example usage in your migration/src/main.rs:");
    println!("");
    println!("  use sea_orm_migration_spanner::{{SpannerMigratorTrait, SpannerMigrationTrait}};");
    println!("");
    println!("  struct Migrator;");
    println!("");
    println!("  impl SpannerMigratorTrait for Migrator {{");
    println!("      fn migrations() -> Vec<Box<dyn SpannerMigrationTrait>> {{");
    println!("          vec![Box::new(m20220101_000001_create_table::Migration)]");
    println!("      }}");
    println!("  }}");
    println!("");
    println!("  #[tokio::main]");
    println!("  async fn main() {{");
    println!("      Migrator::up(\"projects/.../databases/...\", None).await.unwrap();");
    println!("  }}");
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("CLI feature not enabled. Rebuild with --features cli");
}
