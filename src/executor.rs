use {
    crate::{
        error::{SpannerDbErr, SpannerTxError},
        query_result::SpannerQueryResult,
    },
    gcloud_spanner::client::Client,
    sea_orm::{DbErr, Statement},
    std::sync::Arc,
};

pub struct SpannerExecutor {
    client: Arc<Client>,
}

impl SpannerExecutor {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn execute(&self, stmt: Statement) -> Result<i64, DbErr> {
        let spanner_stmt = crate::bind::convert_statement(&stmt)?;

        let result = self
            .client
            .read_write_transaction(|tx| {
                let stmt = spanner_stmt.clone();
                Box::pin(async move { tx.update(stmt).await.map_err(SpannerTxError::from) })
            })
            .await
            .map_err(|e: crate::error::SpannerTxError| SpannerDbErr::Execution(e.to_string()))?;

        Ok(result.1)
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<SpannerQueryResult>, DbErr> {
        let results = self.query_all(stmt).await?;
        Ok(results.into_iter().next())
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<SpannerQueryResult>, DbErr> {
        let spanner_stmt = crate::bind::convert_statement(&stmt)?;

        let mut tx = self
            .client
            .single()
            .await
            .map_err(|e| SpannerDbErr::Query(e.to_string()))?;

        let mut iter = tx
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
