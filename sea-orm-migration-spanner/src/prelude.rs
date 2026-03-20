//! Prelude module for sea-orm-migration-spanner
//!
//! Import this to get all commonly used types and traits:
//! ```rust
//! use sea_orm_migration_spanner::prelude::*;
//! ```

pub use {
    crate::{
        cli,
        migrator::{MigrationName, MigrationStatus, MigrationTrait, MigratorTrait},
        schema_manager::SchemaManager,
        sea_orm_migration,
    },
    async_trait::async_trait,
    sea_orm::{
        sea_query::{self, *},
        ConnectionTrait, DatabaseConnection, DbErr, DeriveIden, ExecResult,
    },
    sea_orm_migration::prelude::DeriveMigrationName,
    sea_query_spanner::{SpannerAlterTable, SpannerIndexBuilder, SpannerTableBuilder},
};
