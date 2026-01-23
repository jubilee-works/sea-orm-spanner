use crate::error::SpannerDbErr;
use crate::query_result::SpannerQueryResult;
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::statement::Statement as SpannerStatement;
use google_cloud_spanner::transaction_ro::ReadOnlyTransaction;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;
use sea_orm::{DbErr, Statement};

pub struct SpannerReadWriteTransaction<'a> {
    tx: &'a mut ReadWriteTransaction,
}

impl<'a> SpannerReadWriteTransaction<'a> {
    pub fn new(tx: &'a mut ReadWriteTransaction) -> Self {
        Self { tx }
    }

    pub async fn execute(&mut self, stmt: Statement) -> Result<i64, DbErr> {
        let spanner_stmt = self.convert_statement(&stmt)?;
        let rows_affected = self.tx.update(spanner_stmt).await
            .map_err(|e| SpannerDbErr::Execution(e.to_string()))?;
        
        Ok(rows_affected)
    }

    pub async fn query_one(&mut self, stmt: Statement) -> Result<Option<SpannerQueryResult>, DbErr> {
        let results = self.query_all(stmt).await?;
        Ok(results.into_iter().next())
    }

    pub async fn query_all(&mut self, stmt: Statement) -> Result<Vec<SpannerQueryResult>, DbErr> {
        let spanner_stmt = self.convert_statement(&stmt)?;
        
        let mut iter = self.tx
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

    fn convert_statement(&self, stmt: &Statement) -> Result<SpannerStatement, DbErr> {
        let sql = &stmt.sql;
        let mut spanner_stmt = SpannerStatement::new(sql);

        if let Some(values) = &stmt.values {
            for (idx, value) in values.0.iter().enumerate() {
                let param_name = format!("p{}", idx + 1);
                self.bind_value(&mut spanner_stmt, &param_name, value)?;
            }
        }

        Ok(spanner_stmt)
    }

    fn bind_value(
        &self,
        stmt: &mut SpannerStatement,
        param_name: &str,
        value: &sea_orm::Value,
    ) -> Result<(), DbErr> {
        use sea_orm::Value;

        match value {
            Value::Bool(Some(v)) => stmt.add_param(param_name, v),
            Value::Bool(None) => stmt.add_param(param_name, &Option::<bool>::None),
            Value::TinyInt(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::TinyInt(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::SmallInt(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::SmallInt(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::Int(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::Int(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::BigInt(Some(v)) => stmt.add_param(param_name, v),
            Value::BigInt(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::TinyUnsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::TinyUnsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::SmallUnsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::SmallUnsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::Unsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::Unsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::BigUnsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::BigUnsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::Float(Some(v)) => stmt.add_param(param_name, &(*v as f64)),
            Value::Float(None) => stmt.add_param(param_name, &Option::<f64>::None),
            Value::Double(Some(v)) => stmt.add_param(param_name, v),
            Value::Double(None) => stmt.add_param(param_name, &Option::<f64>::None),
            Value::String(Some(v)) => stmt.add_param(param_name, v.as_ref()),
            Value::String(None) => stmt.add_param(param_name, &Option::<String>::None),
            Value::Char(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            Value::Char(None) => stmt.add_param(param_name, &Option::<String>::None),
            Value::Bytes(Some(v)) => stmt.add_param(param_name, v.as_ref()),
            Value::Bytes(None) => stmt.add_param(param_name, &Option::<Vec<u8>>::None),
            
            #[cfg(feature = "with-chrono")]
            Value::ChronoDate(Some(v)) => {
                stmt.add_param(param_name, &v.format("%Y-%m-%d").to_string())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDate(None) => stmt.add_param(param_name, &Option::<String>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoTime(Some(v)) => {
                stmt.add_param(param_name, &v.format("%H:%M:%S%.f").to_string())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoTime(None) => stmt.add_param(param_name, &Option::<String>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(Some(v)) => {
                stmt.add_param(param_name, &v.and_utc())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(Some(v)) => stmt.add_param(param_name, v.as_ref()),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(Some(v)) => {
                stmt.add_param(param_name, &v.to_utc())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(Some(v)) => {
                stmt.add_param(param_name, &v.to_utc())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            
            #[cfg(feature = "with-uuid")]
            Value::Uuid(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(None) => stmt.add_param(param_name, &Option::<String>::None),
            
            #[cfg(feature = "with-json")]
            Value::Json(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            #[cfg(feature = "with-json")]
            Value::Json(None) => stmt.add_param(param_name, &Option::<String>::None),
            
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(Some(v)) => {
                stmt.add_param(param_name, &v.to_string())
            }
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(None) => stmt.add_param(param_name, &Option::<String>::None),
            
            #[allow(unreachable_patterns)]
            _ => {
                return Err(SpannerDbErr::TypeConversion {
                    column: param_name.to_string(),
                    expected: "supported type".to_string(),
                    got: format!("{:?}", value),
                }.into());
            }
        }
        Ok(())
    }
}

pub struct SpannerReadOnlyTransaction<'a> {
    tx: &'a mut ReadOnlyTransaction,
}

impl<'a> SpannerReadOnlyTransaction<'a> {
    pub fn new(tx: &'a mut ReadOnlyTransaction) -> Self {
        Self { tx }
    }

    pub async fn query_one(&mut self, stmt: Statement) -> Result<Option<SpannerQueryResult>, DbErr> {
        let results = self.query_all(stmt).await?;
        Ok(results.into_iter().next())
    }

    pub async fn query_all(&mut self, stmt: Statement) -> Result<Vec<SpannerQueryResult>, DbErr> {
        let spanner_stmt = self.convert_statement(&stmt)?;
        
        let mut iter = self.tx
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

    fn convert_statement(&self, stmt: &Statement) -> Result<SpannerStatement, DbErr> {
        let sql = &stmt.sql;
        let mut spanner_stmt = SpannerStatement::new(sql);

        if let Some(values) = &stmt.values {
            for (idx, value) in values.0.iter().enumerate() {
                let param_name = format!("p{}", idx + 1);
                self.bind_value(&mut spanner_stmt, &param_name, value)?;
            }
        }

        Ok(spanner_stmt)
    }

    fn bind_value(
        &self,
        stmt: &mut SpannerStatement,
        param_name: &str,
        value: &sea_orm::Value,
    ) -> Result<(), DbErr> {
        use sea_orm::Value;

        match value {
            Value::Bool(Some(v)) => stmt.add_param(param_name, v),
            Value::Bool(None) => stmt.add_param(param_name, &Option::<bool>::None),
            Value::TinyInt(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::TinyInt(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::SmallInt(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::SmallInt(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::Int(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::Int(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::BigInt(Some(v)) => stmt.add_param(param_name, v),
            Value::BigInt(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::TinyUnsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::TinyUnsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::SmallUnsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::SmallUnsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::Unsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::Unsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::BigUnsigned(Some(v)) => stmt.add_param(param_name, &(*v as i64)),
            Value::BigUnsigned(None) => stmt.add_param(param_name, &Option::<i64>::None),
            Value::Float(Some(v)) => stmt.add_param(param_name, &(*v as f64)),
            Value::Float(None) => stmt.add_param(param_name, &Option::<f64>::None),
            Value::Double(Some(v)) => stmt.add_param(param_name, v),
            Value::Double(None) => stmt.add_param(param_name, &Option::<f64>::None),
            Value::String(Some(v)) => stmt.add_param(param_name, v.as_ref()),
            Value::String(None) => stmt.add_param(param_name, &Option::<String>::None),
            Value::Char(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            Value::Char(None) => stmt.add_param(param_name, &Option::<String>::None),
            Value::Bytes(Some(v)) => stmt.add_param(param_name, v.as_ref()),
            Value::Bytes(None) => stmt.add_param(param_name, &Option::<Vec<u8>>::None),
            
            #[cfg(feature = "with-chrono")]
            Value::ChronoDate(Some(v)) => {
                stmt.add_param(param_name, &v.format("%Y-%m-%d").to_string())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDate(None) => stmt.add_param(param_name, &Option::<String>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoTime(Some(v)) => {
                stmt.add_param(param_name, &v.format("%H:%M:%S%.f").to_string())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoTime(None) => stmt.add_param(param_name, &Option::<String>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(Some(v)) => {
                stmt.add_param(param_name, &v.and_utc())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(Some(v)) => stmt.add_param(param_name, v.as_ref()),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(Some(v)) => {
                stmt.add_param(param_name, &v.to_utc())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(Some(v)) => {
                stmt.add_param(param_name, &v.to_utc())
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(None) => stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None),
            
            #[cfg(feature = "with-uuid")]
            Value::Uuid(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(None) => stmt.add_param(param_name, &Option::<String>::None),
            
            #[cfg(feature = "with-json")]
            Value::Json(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            #[cfg(feature = "with-json")]
            Value::Json(None) => stmt.add_param(param_name, &Option::<String>::None),
            
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(Some(v)) => {
                stmt.add_param(param_name, &v.to_string())
            }
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(None) => stmt.add_param(param_name, &Option::<String>::None),
            
            #[allow(unreachable_patterns)]
            _ => {
                return Err(SpannerDbErr::TypeConversion {
                    column: param_name.to_string(),
                    expected: "supported type".to_string(),
                    got: format!("{:?}", value),
                }.into());
            }
        }
        Ok(())
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
