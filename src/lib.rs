pub mod array_support;
mod bind;
#[cfg(feature = "with-chrono")]
pub mod chrono_support;
mod database;
mod error;
#[cfg(feature = "with-json")]
pub mod json_support;
mod proxy;
#[cfg(feature = "with-uuid")]
pub mod uuid_support;

#[cfg(feature = "with-json")]
pub use json_support::{SpannerJson, SpannerOptionalJson};
#[cfg(feature = "with-uuid")]
pub use uuid_support::SpannerUuid;
pub use {
    array_support::*,
    database::{
        ensure_database, ensure_instance, ensure_tls, CreateOptions, DatabaseDialect, DatabasePath,
        InstanceConfig, SpannerDatabase,
    },
    error::SpannerDbErr,
    gcloud_spanner::client::ClientConfig,
    sea_orm::{
        entity::prelude::*, ActiveModelBehavior, ActiveModelTrait, ConnectionTrait,
        DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder,
        QuerySelect, Set, Unchanged,
    },
    sea_query_spanner::SpannerQueryBuilder,
};
