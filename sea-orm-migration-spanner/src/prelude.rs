//! Prelude module for sea-orm-migration-spanner
//!
//! Import this to get all commonly used types and traits:
//! ```rust
//! use sea_orm_migration_spanner::prelude::*;
//! ```

pub use crate::cli;
pub use crate::migrator::{MigrationName, MigrationStatus, MigrationTrait, MigratorTrait};
pub use crate::schema_manager::SchemaManager;
pub use crate::sea_orm_migration;
pub use async_trait::async_trait;
pub use sea_orm::{
    sea_query::{self, *},
    ConnectionTrait, DbErr, DeriveIden,
};
pub use sea_orm_migration::prelude::DeriveMigrationName;
pub use sea_query_spanner::{SpannerAlterTable, SpannerIndexBuilder, SpannerTableBuilder};
