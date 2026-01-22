//! Prelude module for sea-orm-migration-spanner
//!
//! Import this to get all commonly used types and traits:
//! ```rust
//! use sea_orm_migration_spanner::prelude::*;
//! ```

pub use crate::migrator::{MigrationName, MigrationStatus, MigrationTrait, MigratorTrait};
pub use crate::schema_manager::SchemaManager;
pub use async_trait::async_trait;
pub use sea_orm::{
    self,
    sea_query::{self, *},
    ConnectionTrait, DbErr, DeriveIden, DeriveMigrationName,
};
pub use sea_query_spanner::{SpannerAlterTable, SpannerIndexBuilder, SpannerTableBuilder};
