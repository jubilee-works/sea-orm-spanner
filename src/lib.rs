pub mod array_support;
#[cfg(feature = "with-chrono")]
pub mod chrono_support;
mod database;
mod error;
#[cfg(feature = "with-json")]
pub mod json_support;
mod proxy;
#[cfg(feature = "with-uuid")]
pub mod uuid_support;

pub use array_support::*;
pub use database::{
    ensure_database, ensure_instance, CreateOptions, DatabaseDialect, DatabasePath, InstanceConfig,
    SpannerDatabase,
};
pub use error::SpannerDbErr;
#[cfg(feature = "with-json")]
pub use json_support::{SpannerJson, SpannerOptionalJson};
#[cfg(feature = "with-uuid")]
pub use uuid_support::SpannerUuid;

pub use sea_query_spanner::SpannerQueryBuilder;

pub use sea_orm::{
    entity::prelude::*, ActiveModelBehavior, ActiveModelTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, Set, Unchanged,
};
