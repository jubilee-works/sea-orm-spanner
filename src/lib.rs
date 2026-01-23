mod database;
mod error;
mod proxy;
#[cfg(feature = "with-uuid")]
pub mod uuid_support;

pub use database::SpannerDatabase;
pub use error::SpannerDbErr;
#[cfg(feature = "with-uuid")]
pub use uuid_support::SpannerUuid;

pub use sea_query_spanner::SpannerQueryBuilder;

pub use sea_orm::{
    entity::prelude::*, ActiveModelBehavior, ActiveModelTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, Set, Unchanged,
};
