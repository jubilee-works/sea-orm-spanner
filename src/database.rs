use crate::error::SpannerDbErr;
use crate::proxy::SpannerProxy;
use google_cloud_spanner::client::{Client, ClientConfig};
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::sync::Arc;

pub struct SpannerDatabase;

impl SpannerDatabase {
    pub async fn connect(database: &str) -> Result<DatabaseConnection, DbErr> {
        let client = Client::new(database)
            .await
            .map_err(|e| SpannerDbErr::Connection(e.to_string()))?;

        let proxy = SpannerProxy::new(Arc::new(client));
        Database::connect_proxy(sea_orm::DbBackend::MySql, Arc::new(Box::new(proxy))).await
    }

    pub async fn connect_with_config(
        database: &str,
        config: ClientConfig,
    ) -> Result<DatabaseConnection, DbErr> {
        let client = Client::new_with_config(database, config)
            .await
            .map_err(|e| SpannerDbErr::Connection(e.to_string()))?;

        let proxy = SpannerProxy::new(Arc::new(client));
        Database::connect_proxy(sea_orm::DbBackend::MySql, Arc::new(Box::new(proxy))).await
    }

    pub async fn connect_with_emulator(database: &str) -> Result<DatabaseConnection, DbErr> {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        Self::connect(database).await
    }

    pub async fn connect_with_emulator_host(
        database: &str,
        emulator_host: &str,
    ) -> Result<DatabaseConnection, DbErr> {
        std::env::set_var("SPANNER_EMULATOR_HOST", emulator_host);
        Self::connect(database).await
    }
}
