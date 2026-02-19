use sea_orm_migration_spanner::prelude::*;

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    cli::run_cli(migration::Migrator).await;
}
