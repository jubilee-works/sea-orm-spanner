use crate::connection::SpannerConnection;
use crate::error::SpannerDbErr;
use google_cloud_spanner::client::{Client, ClientConfig};
use sea_orm::DbErr;
use std::sync::Arc;

pub struct SpannerDatabase;

impl SpannerDatabase {
    pub async fn connect(database: &str) -> Result<SpannerConnection, DbErr> {
        let client = Client::new(database)
            .await
            .map_err(|e| SpannerDbErr::Connection(e.to_string()))?;

        Ok(SpannerConnection::new(Arc::new(client)))
    }

    pub async fn connect_with_config(
        database: &str,
        config: ClientConfig,
    ) -> Result<SpannerConnection, DbErr> {
        let client = Client::new_with_config(database, config)
            .await
            .map_err(|e| SpannerDbErr::Connection(e.to_string()))?;

        Ok(SpannerConnection::new(Arc::new(client)))
    }

    pub async fn connect_with_emulator(database: &str) -> Result<SpannerConnection, DbErr> {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        Self::connect(database).await
    }

    pub async fn connect_with_emulator_host(
        database: &str,
        emulator_host: &str,
    ) -> Result<SpannerConnection, DbErr> {
        std::env::set_var("SPANNER_EMULATOR_HOST", emulator_host);
        Self::connect(database).await
    }
}
