use crate::error::SpannerDbErr;
use crate::query_result::SpannerQueryResult;
use gcloud_spanner::transaction_ro::ReadOnlyTransaction;
use gcloud_spanner::transaction_rw::ReadWriteTransaction;
use sea_orm::{DbErr, Statement};

pub struct SpannerReadWriteTransaction<'a> {
    tx: &'a mut ReadWriteTransaction,
}

impl<'a> SpannerReadWriteTransaction<'a> {
    pub fn new(tx: &'a mut ReadWriteTransaction) -> Self {
        Self { tx }
    }

    pub async fn execute(&mut self, stmt: Statement) -> Result<i64, DbErr> {
        let spanner_stmt = crate::bind::convert_statement(&stmt)?;
        let rows_affected = self
            .tx
            .update(spanner_stmt)
            .await
            .map_err(|e| SpannerDbErr::Execution(e.to_string()))?;

        Ok(rows_affected)
    }

    pub async fn query_one(
        &mut self,
        stmt: Statement,
    ) -> Result<Option<SpannerQueryResult>, DbErr> {
        let results = self.query_all(stmt).await?;
        Ok(results.into_iter().next())
    }

    pub async fn query_all(&mut self, stmt: Statement) -> Result<Vec<SpannerQueryResult>, DbErr> {
        let spanner_stmt = crate::bind::convert_statement(&stmt)?;

        let mut iter = self
            .tx
            .query(spanner_stmt)
            .await
            .map_err(|e| SpannerDbErr::Query(e.to_string()))?;

        let mut results = Vec::new();
        while let Some(row) = iter
            .next()
            .await
            .map_err(|e| SpannerDbErr::Query(e.to_string()))?
        {
            results.push(SpannerQueryResult::new(row));
        }

        Ok(results)
    }
}

pub struct SpannerReadOnlyTransaction<'a> {
    tx: &'a mut ReadOnlyTransaction,
}

impl<'a> SpannerReadOnlyTransaction<'a> {
    pub fn new(tx: &'a mut ReadOnlyTransaction) -> Self {
        Self { tx }
    }

    pub async fn query_one(
        &mut self,
        stmt: Statement,
    ) -> Result<Option<SpannerQueryResult>, DbErr> {
        let results = self.query_all(stmt).await?;
        Ok(results.into_iter().next())
    }

    pub async fn query_all(&mut self, stmt: Statement) -> Result<Vec<SpannerQueryResult>, DbErr> {
        let spanner_stmt = crate::bind::convert_statement(&stmt)?;

        let mut iter = self
            .tx
            .query(spanner_stmt)
            .await
            .map_err(|e| SpannerDbErr::Query(e.to_string()))?;

        let mut results = Vec::new();
        while let Some(row) = iter
            .next()
            .await
            .map_err(|e| SpannerDbErr::Query(e.to_string()))?
        {
            results.push(SpannerQueryResult::new(row));
        }

        Ok(results)
    }
}

impl std::fmt::Debug for SpannerReadWriteTransaction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpannerReadWriteTransaction").finish()
    }
}

impl std::fmt::Debug for SpannerReadOnlyTransaction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpannerReadOnlyTransaction").finish()
    }
}
