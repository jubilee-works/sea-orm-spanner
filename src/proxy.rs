use crate::error::SpannerDbErr;
use async_trait::async_trait;
use google_cloud_spanner::client::Client;
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::statement::Statement as SpannerStatement;
use sea_orm::ProxyDatabaseTrait;
use sea_orm::ProxyExecResult;
use sea_orm::ProxyRow;
use sea_orm::{DbErr, Statement};
use sea_query::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::SpannerTxError;

pub struct SpannerProxy {
    client: Arc<Client>,
}

impl std::fmt::Debug for SpannerProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpannerProxy").finish()
    }
}

impl SpannerProxy {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    fn convert_statement(&self, stmt: &Statement) -> Result<SpannerStatement, DbErr> {
        let sql = Self::rewrite_placeholders(&stmt.sql);
        let sql = Self::rewrite_mysql_quotes(&sql);
        let mut spanner_stmt = SpannerStatement::new(&sql);

        if let Some(values) = &stmt.values {
            for (idx, value) in values.0.iter().enumerate() {
                let param_name = format!("p{}", idx + 1);
                self.bind_value(&mut spanner_stmt, &param_name, value)?;
            }
        }

        Ok(spanner_stmt)
    }

    fn rewrite_placeholders(sql: &str) -> String {
        let mut result = String::with_capacity(sql.len() + 50);
        let mut param_idx = 1;
        let mut chars = sql.chars().peekable();
        let mut in_string = false;
        let mut string_char = ' ';

        while let Some(c) = chars.next() {
            if !in_string && (c == '\'' || c == '"') {
                in_string = true;
                string_char = c;
                result.push(c);
            } else if in_string && c == string_char {
                if chars.peek() == Some(&string_char) {
                    result.push(c);
                    result.push(chars.next().unwrap());
                } else {
                    in_string = false;
                    result.push(c);
                }
            } else if !in_string && c == '?' {
                result.push_str(&format!("@p{}", param_idx));
                param_idx += 1;
            } else {
                result.push(c);
            }
        }

        result
    }

    fn rewrite_mysql_quotes(sql: &str) -> String {
        sql.replace('`', "")
    }

    fn bind_value(
        &self,
        stmt: &mut SpannerStatement,
        param_name: &str,
        value: &Value,
    ) -> Result<(), DbErr> {
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
            Value::ChronoDateTime(Some(v)) => stmt.add_param(param_name, &v.and_utc()),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(None) => {
                stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None)
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(Some(v)) => stmt.add_param(param_name, v.as_ref()),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(None) => {
                stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None)
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(Some(v)) => stmt.add_param(param_name, &v.to_utc()),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(None) => {
                stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None)
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(Some(v)) => stmt.add_param(param_name, &v.to_utc()),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(None) => {
                stmt.add_param(param_name, &Option::<chrono::DateTime<chrono::Utc>>::None)
            }

            #[cfg(feature = "with-uuid")]
            Value::Uuid(Some(v)) => stmt.add_param(param_name, &crate::SpannerUuid::from(*v.as_ref())),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(None) => stmt.add_param(param_name, &crate::uuid_support::SpannerOptionalUuid::none()),

            #[cfg(feature = "with-json")]
            Value::Json(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            #[cfg(feature = "with-json")]
            Value::Json(None) => stmt.add_param(param_name, &Option::<String>::None),

            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(None) => stmt.add_param(param_name, &Option::<String>::None),

            #[allow(unreachable_patterns)]
            _ => {
                return Err(SpannerDbErr::TypeConversion {
                    column: param_name.to_string(),
                    expected: "supported type".to_string(),
                    got: format!("{:?}", value),
                }
                .into());
            }
        }

        Ok(())
    }

    fn spanner_value_to_sea_value(
        row: &google_cloud_spanner::row::Row,
        idx: usize,
        column_name: &str,
    ) -> Value {
        if let Ok(v) = row.column::<Option<bool>>(idx) {
            return Value::Bool(v);
        }
        if let Ok(v) = row.column::<Option<i64>>(idx) {
            if let Some(val) = v {
                if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
                    return Value::Int(Some(val as i32));
                }
            }
            return Value::BigInt(v);
        }
        if let Ok(v) = row.column::<Option<f64>>(idx) {
            return Value::Double(v);
        }
        #[cfg(feature = "with-chrono")]
        if let Ok(v) = row.column::<Option<chrono::DateTime<chrono::Utc>>>(idx) {
            return Value::ChronoDateTimeUtc(v.map(Box::new));
        }
        if let Ok(v) = row.column::<Option<String>>(idx) {
            return Value::String(v.map(Box::new));
        }
        if let Ok(v) = row.column::<Option<Vec<u8>>>(idx) {
            return Value::Bytes(v.map(Box::new));
        }

        tracing::warn!("Unknown column type for {}, returning null", column_name);
        Value::String(None)
    }

    fn extract_column_names_from_statement(statement: &Statement) -> Vec<String> {
        let sql = statement.sql.to_uppercase();
        if let Some(select_pos) = sql.find("SELECT") {
            if let Some(from_pos) = sql.find("FROM") {
                let columns_part = &statement.sql[select_pos + 6..from_pos];
                return columns_part
                    .split(',')
                    .map(|s| {
                        let s = s.trim();
                        if let Some(as_pos) = s.to_uppercase().rfind(" AS ") {
                            s[as_pos + 4..]
                                .trim()
                                .trim_matches('"')
                                .trim_matches('`')
                                .to_string()
                        } else if s.contains('.') {
                            s.rsplit('.')
                                .next()
                                .unwrap_or(s)
                                .trim()
                                .trim_matches('"')
                                .trim_matches('`')
                                .to_string()
                        } else {
                            s.trim_matches('"').trim_matches('`').to_string()
                        }
                    })
                    .filter(|s| !s.is_empty() && s != "*")
                    .collect();
            }
        }
        Vec::new()
    }
}

#[async_trait]
impl ProxyDatabaseTrait for SpannerProxy {
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let spanner_stmt = self.convert_statement(&statement)?;

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
        let mut column_names: Option<Vec<String>> = None;

        while let Some(row) = iter
            .next()
            .await
            .map_err(|e| SpannerDbErr::Query(e.to_string()))?
        {
            if column_names.is_none() {
                column_names = Some(Self::extract_column_names_from_statement(&statement));
            }

            let col_names = column_names.as_ref().unwrap();
            let mut values = BTreeMap::new();

            for (idx, col_name) in col_names.iter().enumerate() {
                let value = Self::spanner_value_to_sea_value(&row, idx, col_name);
                values.insert(col_name.clone(), value);
            }

            if values.is_empty() {
                for idx in 0..100 {
                    let col_name = format!("col_{}", idx);
                    if let Ok(v) = row.column::<Option<String>>(idx) {
                        values.insert(col_name, Value::String(v.map(Box::new)));
                    } else {
                        break;
                    }
                }
            }

            results.push(ProxyRow { values });
        }

        Ok(results)
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let spanner_stmt = self.convert_statement(&statement)?;

        let result = self
            .client
            .read_write_transaction(|tx, _cancel| {
                let stmt = spanner_stmt.clone();
                Box::pin(async move { tx.update(stmt).await.map_err(SpannerTxError::from) })
            })
            .await
            .map_err(|e| SpannerDbErr::Execution(e.to_string()))?;

        Ok(ProxyExecResult {
            last_insert_id: 0,
            rows_affected: result.1 as u64,
        })
    }

    async fn begin(&self) {
        // Spanner uses callback-based transactions, handled differently
    }

    async fn commit(&self) {
        // Handled by transaction callback
    }

    async fn rollback(&self) {
        // Handled by transaction callback
    }

    async fn ping(&self) -> Result<(), DbErr> {
        let stmt = SpannerStatement::new("SELECT 1");
        let mut tx = self
            .client
            .single()
            .await
            .map_err(|e| SpannerDbErr::Connection(e.to_string()))?;

        tx.query(stmt)
            .await
            .map_err(|e| SpannerDbErr::Connection(e.to_string()))?;

        Ok(())
    }
}
