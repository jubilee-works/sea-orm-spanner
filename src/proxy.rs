use crate::array_support::*;
use crate::error::SpannerDbErr;
use async_trait::async_trait;
use gcloud_googleapis::spanner::v1::TypeCode;
use gcloud_spanner::client::Client;

use gcloud_spanner::statement::Statement as SpannerStatement;
use sea_orm::ProxyDatabaseTrait;
use sea_orm::ProxyExecResult;
use sea_orm::ProxyRow;
use sea_orm::{DbErr, Statement};
use sea_query::{ArrayType, Value};
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

            Value::String(Some(v)) => stmt.add_param(param_name, v),
            Value::String(None) => stmt.add_param(param_name, &Option::<String>::None),

            Value::Char(Some(v)) => stmt.add_param(param_name, &v.to_string()),
            Value::Char(None) => stmt.add_param(param_name, &Option::<String>::None),

            Value::Bytes(Some(v)) => stmt.add_param(param_name, v),
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
            Value::ChronoDateTime(Some(v)) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerNaiveDateTime::new(*v),
            ),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(None) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerOptionalNaiveDateTime::none(),
            ),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(Some(v)) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerTimestamp::new(*v),
            ),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(None) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerOptionalTimestamp::none(),
            ),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(Some(v)) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerTimestamp::new(v.to_utc()),
            ),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(None) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerOptionalTimestamp::none(),
            ),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(Some(v)) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerTimestamp::new(v.to_utc()),
            ),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(None) => stmt.add_param(
                param_name,
                &crate::chrono_support::SpannerOptionalTimestamp::none(),
            ),

            #[cfg(feature = "with-uuid")]
            Value::Uuid(Some(v)) => stmt.add_param(param_name, v),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(None) => stmt.add_param(param_name, &Option::<uuid::Uuid>::None),

            #[cfg(feature = "with-json")]
            Value::Json(Some(v)) => stmt.add_param(
                param_name,
                &crate::json_support::SpannerOptionalJson::some(v.as_ref().clone()),
            ),
            #[cfg(feature = "with-json")]
            Value::Json(None) => stmt.add_param(
                param_name,
                &crate::json_support::SpannerOptionalJson::none(),
            ),

            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(Some(v)) => {
                use std::str::FromStr;
                let big_decimal = gcloud_spanner::bigdecimal::BigDecimal::from_str(&v.to_string())
                    .map_err(|e| SpannerDbErr::TypeConversion {
                        column: param_name.to_string(),
                        expected: "BigDecimal".to_string(),
                        got: format!("{}: {}", v, e),
                    })?;
                stmt.add_param(param_name, &big_decimal);
            }
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(None) => stmt.add_param(
                param_name,
                &Option::<gcloud_spanner::bigdecimal::BigDecimal>::None,
            ),

            Value::Array(array_type, Some(values)) => {
                self.bind_array(stmt, param_name, array_type, values)?;
                return Ok(());
            }
            Value::Array(array_type, None) => {
                self.bind_null_array(stmt, param_name, array_type)?;
                return Ok(());
            }

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

    fn bind_array(
        &self,
        stmt: &mut SpannerStatement,
        param_name: &str,
        array_type: &ArrayType,
        values: &[Value],
    ) -> Result<(), DbErr> {
        match array_type {
            ArrayType::Bool => {
                let arr: Vec<bool> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::Bool(Some(b)) => Some(*b),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            ArrayType::TinyInt | ArrayType::SmallInt | ArrayType::Int | ArrayType::BigInt => {
                let arr: Vec<i64> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::TinyInt(Some(i)) => Some(*i as i64),
                        Value::SmallInt(Some(i)) => Some(*i as i64),
                        Value::Int(Some(i)) => Some(*i as i64),
                        Value::BigInt(Some(i)) => Some(*i),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            ArrayType::TinyUnsigned
            | ArrayType::SmallUnsigned
            | ArrayType::Unsigned
            | ArrayType::BigUnsigned => {
                let arr: Vec<i64> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::TinyUnsigned(Some(i)) => Some(*i as i64),
                        Value::SmallUnsigned(Some(i)) => Some(*i as i64),
                        Value::Unsigned(Some(i)) => Some(*i as i64),
                        Value::BigUnsigned(Some(i)) => Some(*i as i64),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            ArrayType::Float | ArrayType::Double => {
                let arr: Vec<f64> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::Float(Some(f)) => Some(*f as f64),
                        Value::Double(Some(d)) => Some(*d),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            ArrayType::String | ArrayType::Char => {
                let arr: Vec<String> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::String(Some(s)) => Some(s.clone()),
                        Value::Char(Some(c)) => Some(c.to_string()),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            ArrayType::Bytes => {
                let arr: Vec<Vec<u8>> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::Bytes(Some(b)) => Some(b.clone()),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &SpannerBytesArray(arr));
            }
            #[cfg(feature = "with-chrono")]
            ArrayType::ChronoDate
            | ArrayType::ChronoTime
            | ArrayType::ChronoDateTime
            | ArrayType::ChronoDateTimeUtc
            | ArrayType::ChronoDateTimeLocal
            | ArrayType::ChronoDateTimeWithTimeZone => {
                let arr: Vec<String> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::ChronoDate(Some(d)) => Some(d.format("%Y-%m-%d").to_string()),
                        Value::ChronoTime(Some(t)) => Some(t.format("%H:%M:%S%.f").to_string()),
                        Value::ChronoDateTime(Some(dt)) => {
                            Some(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        Value::ChronoDateTimeUtc(Some(dt)) => {
                            Some(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        Value::ChronoDateTimeLocal(Some(dt)) => {
                            Some(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        Value::ChronoDateTimeWithTimeZone(Some(dt)) => {
                            Some(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            #[cfg(feature = "with-uuid")]
            ArrayType::Uuid => {
                let arr: Vec<String> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::Uuid(Some(u)) => Some(u.to_string()),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            #[cfg(feature = "with-json")]
            ArrayType::Json => {
                let arr: Vec<String> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::Json(Some(j)) => Some(j.to_string()),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            #[cfg(feature = "with-rust_decimal")]
            ArrayType::Decimal => {
                let arr: Vec<String> = values
                    .iter()
                    .filter_map(|v| match v {
                        Value::Decimal(Some(d)) => Some(d.to_string()),
                        _ => None,
                    })
                    .collect();
                stmt.add_param(param_name, &arr);
            }
            #[allow(unreachable_patterns)]
            _ => {
                return Err(SpannerDbErr::TypeConversion {
                    column: param_name.to_string(),
                    expected: "supported array type".to_string(),
                    got: format!("{:?}", array_type),
                }
                .into());
            }
        }
        Ok(())
    }

    fn bind_null_array(
        &self,
        stmt: &mut SpannerStatement,
        param_name: &str,
        array_type: &ArrayType,
    ) -> Result<(), DbErr> {
        match array_type {
            ArrayType::Bool => {
                stmt.add_param(param_name, &SpannerOptionalBoolArray::none());
            }
            ArrayType::TinyInt
            | ArrayType::SmallInt
            | ArrayType::Int
            | ArrayType::BigInt
            | ArrayType::TinyUnsigned
            | ArrayType::SmallUnsigned
            | ArrayType::Unsigned
            | ArrayType::BigUnsigned => {
                stmt.add_param(param_name, &SpannerOptionalInt64Array::none());
            }
            ArrayType::Float | ArrayType::Double => {
                stmt.add_param(param_name, &SpannerOptionalFloat64Array::none());
            }
            ArrayType::String | ArrayType::Char => {
                stmt.add_param(param_name, &SpannerOptionalStringArray::none());
            }
            ArrayType::Bytes => {
                stmt.add_param(param_name, &SpannerOptionalBytesArray::none());
            }
            #[allow(unreachable_patterns)]
            _ => {
                stmt.add_param(param_name, &SpannerOptionalStringArray::none());
            }
        }
        Ok(())
    }

    fn spanner_value_to_sea_value(
        row: &gcloud_spanner::row::Row,
        idx: usize,
        column_name: &str,
    ) -> Value {
        let type_code = row
            .field(idx)
            .and_then(|f| f.r#type.as_ref())
            .map(|t| t.code)
            .unwrap_or(0);

        match TypeCode::try_from(type_code) {
            Ok(TypeCode::Bool) => match row.column::<Option<bool>>(idx) {
                Ok(v) => return Value::Bool(v),
                Err(e) => {
                    tracing::warn!(
                        "Failed to read BOOL column {} at index {}: {:?}",
                        column_name,
                        idx,
                        e
                    );
                }
            },
            Ok(TypeCode::Int64) => match row.column::<Option<i64>>(idx) {
                Ok(v) => return Value::BigInt(v),
                Err(e) => {
                    tracing::warn!(
                        "Failed to read INT64 column {} at index {}: {:?}",
                        column_name,
                        idx,
                        e
                    );
                }
            },
            Ok(TypeCode::Float64 | TypeCode::Float32) => match row.column::<Option<f64>>(idx) {
                Ok(v) => return Value::Double(v),
                Err(e) => {
                    tracing::warn!(
                        "Failed to read FLOAT64 column {} at index {}: {:?}",
                        column_name,
                        idx,
                        e
                    );
                }
            },
            Ok(TypeCode::String) => match row.column::<Option<String>>(idx) {
                Ok(v) => return Value::String(v),
                Err(e) => {
                    tracing::warn!(
                        "Failed to read STRING column {} at index {}: {:?}",
                        column_name,
                        idx,
                        e
                    );
                }
            },
            Ok(TypeCode::Bytes) => match row.column::<Option<Vec<u8>>>(idx) {
                Ok(v) => return Value::Bytes(v),
                Err(e) => {
                    tracing::warn!(
                        "Failed to read BYTES column {} at index {}: {:?}",
                        column_name,
                        idx,
                        e
                    );
                }
            },
            Ok(TypeCode::Timestamp) => {
                #[cfg(feature = "with-chrono")]
                {
                    match row.column::<Option<time::OffsetDateTime>>(idx) {
                        Ok(v) => {
                            if let Some(odt) = v {
                                let chrono_dt = chrono::DateTime::from_timestamp(
                                    odt.unix_timestamp(),
                                    odt.nanosecond(),
                                )
                                .unwrap_or(chrono::DateTime::UNIX_EPOCH);
                                return Value::ChronoDateTime(Some(chrono_dt.naive_utc()));
                            }
                            return Value::ChronoDateTime(None);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to read timestamp column {} as OffsetDateTime: {:?}",
                                column_name,
                                e
                            );
                        }
                    }
                }
                #[cfg(not(feature = "with-chrono"))]
                if let Ok(v) = row.column::<Option<String>>(idx) {
                    return Value::String(v);
                }
            }
            Ok(TypeCode::Date) => {
                #[cfg(feature = "with-chrono")]
                if let Ok(v) = row.column::<Option<time::Date>>(idx) {
                    if let Some(d) = v {
                        let naive_date = chrono::NaiveDate::from_ymd_opt(
                            d.year(),
                            d.month() as u32,
                            d.day() as u32,
                        );
                        return Value::ChronoDate(naive_date);
                    }
                    return Value::ChronoDate(None);
                }
                #[cfg(not(feature = "with-chrono"))]
                if let Ok(v) = row.column::<Option<String>>(idx) {
                    return Value::String(v);
                }
            }
            Ok(TypeCode::Numeric) => {
                #[cfg(feature = "with-rust_decimal")]
                if let Ok(Some(big_decimal)) =
                    row.column::<Option<gcloud_spanner::bigdecimal::BigDecimal>>(idx)
                {
                    if let Ok(decimal) =
                        rust_decimal::Decimal::from_str_exact(&big_decimal.to_string())
                    {
                        return Value::Decimal(Some(decimal));
                    }
                }
                #[cfg(not(feature = "with-rust_decimal"))]
                if let Ok(v) = row.column::<Option<String>>(idx) {
                    return Value::String(v);
                }
            }
            Ok(TypeCode::Json) => {
                #[cfg(feature = "with-json")]
                if let Ok(v) = row.column::<Option<String>>(idx) {
                    if let Some(s) = v {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s) {
                            return Value::Json(Some(Box::new(json)));
                        }
                    }
                    return Value::Json(None);
                }
                #[cfg(not(feature = "with-json"))]
                if let Ok(v) = row.column::<Option<String>>(idx) {
                    return Value::String(v);
                }
            }
            Ok(TypeCode::Uuid) => {
                #[cfg(feature = "with-uuid")]
                if let Ok(v) = row.column::<Option<uuid::Uuid>>(idx) {
                    return Value::Uuid(v);
                }
                #[cfg(not(feature = "with-uuid"))]
                if let Ok(v) = row.column::<Option<String>>(idx) {
                    return Value::String(v);
                }
            }
            Ok(TypeCode::Array) => {
                return Self::read_array_value(row, idx, column_name);
            }
            _ => {}
        }

        tracing::debug!(
            "Type code {} for column {} - attempting fallback type detection",
            type_code,
            column_name
        );

        if let Ok(v) = row.column::<Option<i64>>(idx) {
            tracing::debug!("Fallback: read {} as INT64", column_name);
            return Value::BigInt(v);
        }

        if let Ok(v) = row.column::<Option<f64>>(idx) {
            tracing::debug!("Fallback: read {} as FLOAT64", column_name);
            return Value::Double(v);
        }

        if let Ok(v) = row.column::<Option<String>>(idx) {
            tracing::debug!("Fallback: read {} as STRING", column_name);
            return Value::String(v);
        }

        if let Ok(v) = row.column::<Option<bool>>(idx) {
            tracing::debug!("Fallback: read {} as BOOL", column_name);
            return Value::Bool(v);
        }

        tracing::warn!(
            "Unknown column type {} for {} - all fallback attempts failed",
            type_code,
            column_name
        );
        Value::String(None)
    }

    fn read_array_value(row: &gcloud_spanner::row::Row, idx: usize, column_name: &str) -> Value {
        let element_type_code = row
            .field(idx)
            .and_then(|f| f.r#type.as_ref())
            .and_then(|t| t.array_element_type.as_ref())
            .map(|et| et.code)
            .unwrap_or(0);

        match TypeCode::try_from(element_type_code) {
            Ok(TypeCode::Bool) => {
                if let Ok(arr) = row.column::<Vec<bool>>(idx) {
                    let values: Vec<Value> =
                        arr.into_iter().map(|v| Value::Bool(Some(v))).collect();
                    return Value::Array(ArrayType::Bool, Some(Box::new(values)));
                }
            }
            Ok(TypeCode::Int64) => {
                if let Ok(arr) = row.column::<Vec<i64>>(idx) {
                    let values: Vec<Value> =
                        arr.into_iter().map(|v| Value::BigInt(Some(v))).collect();
                    return Value::Array(ArrayType::BigInt, Some(Box::new(values)));
                }
            }
            Ok(TypeCode::Float64 | TypeCode::Float32) => {
                if let Ok(arr) = row.column::<Vec<f64>>(idx) {
                    let values: Vec<Value> =
                        arr.into_iter().map(|v| Value::Double(Some(v))).collect();
                    return Value::Array(ArrayType::Double, Some(Box::new(values)));
                }
            }
            Ok(TypeCode::String) => {
                if let Ok(arr) = row.column::<Vec<String>>(idx) {
                    let values: Vec<Value> =
                        arr.into_iter().map(|v| Value::String(Some(v))).collect();
                    return Value::Array(ArrayType::String, Some(Box::new(values)));
                }
            }
            Ok(TypeCode::Bytes) => {
                if let Ok(arr) = row.column::<Vec<Vec<u8>>>(idx) {
                    let values: Vec<Value> =
                        arr.into_iter().map(|v| Value::Bytes(Some(v))).collect();
                    return Value::Array(ArrayType::Bytes, Some(Box::new(values)));
                }
            }
            _ => {}
        }

        tracing::warn!(
            "Unknown array element type {} for {}",
            element_type_code,
            column_name
        );
        Value::Array(ArrayType::String, None)
    }

    fn extract_column_names_from_statement(statement: &Statement) -> Vec<String> {
        let sql = statement.sql.to_uppercase();
        if let Some(select_pos) = sql.find("SELECT") {
            if let Some(from_pos) = Self::find_top_level_from(&sql, select_pos + 6) {
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

    fn find_top_level_from(sql: &str, start: usize) -> Option<usize> {
        let bytes = sql.as_bytes();
        let mut paren_depth: i32 = 0;
        let mut i = start;

        while i < bytes.len() {
            match bytes[i] {
                b'(' => paren_depth += 1,
                b')' => paren_depth = paren_depth.saturating_sub(1),
                b'F' if paren_depth == 0 => {
                    if sql[i..].starts_with("FROM") {
                        let next_idx = i + 4;
                        if next_idx >= bytes.len()
                            || !bytes[next_idx].is_ascii_alphanumeric()
                            || bytes[next_idx] == b'_'
                        {
                            return Some(i);
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
        None
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
                        values.insert(col_name, Value::String(v));
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
            .read_write_transaction(|tx| {
                let stmt = spanner_stmt.clone();
                Box::pin(async move { tx.update(stmt).await.map_err(SpannerTxError::from) })
            })
            .await
            .map_err(|e: crate::error::SpannerTxError| SpannerDbErr::Execution(e.to_string()))?;

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
