use crate::error::SpannerDbErr;
use crate::executor::SpannerExecutor;
use crate::query_result::SpannerQueryResult;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::TryAs;
use google_cloud_spanner::client::Client;
use google_cloud_spanner::session::SessionError;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;
use google_cloud_spanner::transaction_ro::ReadOnlyTransaction;
use sea_orm::{DbBackend, DbErr, Statement};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub struct SpannerConnection {
    client: Arc<Client>,
    executor: Arc<SpannerExecutor>,
}

impl SpannerConnection {
    pub fn new(client: Arc<Client>) -> Self {
        let executor = Arc::new(SpannerExecutor::new(client.clone()));
        Self { client, executor }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn close(&self) {
        self.client.close().await;
    }

    pub fn get_database_backend(&self) -> DbBackend {
        DbBackend::Postgres
    }

    pub async fn execute(&self, stmt: Statement) -> Result<i64, DbErr> {
        self.executor.execute(stmt).await
    }

    pub async fn execute_unprepared(&self, sql: &str) -> Result<i64, DbErr> {
        let stmt = Statement::from_string(DbBackend::Postgres, sql.to_owned());
        self.execute(stmt).await
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<SpannerQueryResult>, DbErr> {
        self.executor.query_one(stmt).await
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<SpannerQueryResult>, DbErr> {
        self.executor.query_all(stmt).await
    }

    pub fn support_returning(&self) -> bool {
        false
    }

    pub async fn read_write_transaction<F, T, E>(&self, callback: F) -> Result<T, DbErr>
    where
        E: TryAs<Status> + From<SessionError> + From<Status> + ToString,
        F: for<'tx> Fn(
            &'tx mut ReadWriteTransaction,
            Option<CancellationToken>,
        ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'tx>>,
        T: Send,
    {
        let result = self
            .client
            .read_write_transaction(|tx, cancel| callback(tx, cancel))
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(result.1)
    }

    pub async fn read_only_transaction<F, T>(&self, callback: F) -> Result<T, DbErr>
    where
        F: for<'tx> FnOnce(
            &'tx mut ReadOnlyTransaction,
        ) -> Pin<Box<dyn Future<Output = Result<T, DbErr>> + Send + 'tx>> + Send,
        T: Send,
    {
        let mut tx = self
            .client
            .read_only_transaction()
            .await
            .map_err(|e| SpannerDbErr::Transaction(e.to_string()))?;

        callback(&mut tx).await
    }
}

impl std::fmt::Debug for SpannerConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpannerConnection").finish()
    }
}
