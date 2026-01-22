//! Prelude module for sea-orm-migration-spanner
//!
//! Import this to get all commonly used types and traits:
//! ```rust
//! use sea_orm_migration_spanner::prelude::*;
//! ```

pub use crate::migrator::{MigrationName, MigrationStatus, MigrationTrait, MigratorTrait};
pub use crate::schema::SchemaManager;
pub use async_trait::async_trait;
pub use sea_orm::DbErr;
pub use sea_query_spanner::{SpannerAlterTable, SpannerIndexBuilder, SpannerTableBuilder};
