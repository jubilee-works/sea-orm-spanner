use crate::array_support::*;
use crate::error::SpannerDbErr;
use async_trait::async_trait;
use gcloud_googleapis::spanner::v1::struct_type::Field;
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
        let mut result = String::with_capacity(sql.len());
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
                    // Escaped quote inside string literal
                    result.push(c);
                    result.push(chars.next().unwrap());
                } else {
                    in_string = false;
                    result.push(c);
                }
            } else if !in_string && c == '`' {
                // Skip backticks outside of string literals
            } else {
                result.push(c);
            }
        }

        result
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

            Value::BigUnsigned(Some(v)) => {
                let i = i64::try_from(*v).map_err(|_| SpannerDbErr::TypeConversion {
                    column: param_name.to_string(),
                    expected: "i64".to_string(),
                    got: format!("u64 value {} overflows i64", v),
                })?;
                stmt.add_param(param_name, &i);
            }
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
            Value::Uuid(Some(v)) => {
                stmt.add_param(param_name, &crate::uuid_support::SpannerUuid::new(*v))
            }
            #[cfg(feature = "with-uuid")]
            Value::Uuid(None) => stmt.add_param(
                param_name,
                &crate::uuid_support::SpannerOptionalUuid::none(),
            ),

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
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::Bool(Some(b)) => Ok(*b),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Bool".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            ArrayType::TinyInt | ArrayType::SmallInt | ArrayType::Int | ArrayType::BigInt => {
                let arr: Vec<i64> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::TinyInt(Some(i)) => Ok(*i as i64),
                        Value::SmallInt(Some(i)) => Ok(*i as i64),
                        Value::Int(Some(i)) => Ok(*i as i64),
                        Value::BigInt(Some(i)) => Ok(*i),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Int".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            ArrayType::TinyUnsigned
            | ArrayType::SmallUnsigned
            | ArrayType::Unsigned
            | ArrayType::BigUnsigned => {
                let arr: Vec<i64> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::TinyUnsigned(Some(val)) => Ok(*val as i64),
                        Value::SmallUnsigned(Some(val)) => Ok(*val as i64),
                        Value::Unsigned(Some(val)) => Ok(*val as i64),
                        Value::BigUnsigned(Some(val)) => {
                            i64::try_from(*val).map_err(|_| SpannerDbErr::TypeConversion {
                                column: param_name.to_string(),
                                expected: "i64".to_string(),
                                got: format!("element [{}]: u64 value {} overflows i64", i, val),
                            })
                        }
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Unsigned Int".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            ArrayType::Float | ArrayType::Double => {
                let arr: Vec<f64> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::Float(Some(f)) => Ok(*f as f64),
                        Value::Double(Some(d)) => Ok(*d),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Float".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            ArrayType::String | ArrayType::Char => {
                let arr: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::String(Some(s)) => Ok(s.to_string()),
                        Value::Char(Some(c)) => Ok(c.to_string()),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "String".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            ArrayType::Bytes => {
                let arr: Vec<Vec<u8>> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::Bytes(Some(b)) => Ok(b.to_vec()),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Bytes".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
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
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::ChronoDate(Some(d)) => Ok(d.format("%Y-%m-%d").to_string()),
                        Value::ChronoTime(Some(t)) => Ok(t.format("%H:%M:%S%.f").to_string()),
                        Value::ChronoDateTime(Some(dt)) => {
                            Ok(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        Value::ChronoDateTimeUtc(Some(dt)) => {
                            Ok(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        Value::ChronoDateTimeLocal(Some(dt)) => {
                            Ok(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        Value::ChronoDateTimeWithTimeZone(Some(dt)) => {
                            Ok(dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                        }
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Chrono DateTime".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            #[cfg(feature = "with-uuid")]
            ArrayType::Uuid => {
                let arr: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::Uuid(Some(u)) => Ok(u.to_string()),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Uuid".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            #[cfg(feature = "with-json")]
            ArrayType::Json => {
                let arr: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::Json(Some(j)) => Ok(j.to_string()),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Json".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                stmt.add_param(param_name, &arr);
            }
            #[cfg(feature = "with-rust_decimal")]
            ArrayType::Decimal => {
                let arr: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| match v {
                        Value::Decimal(Some(d)) => Ok(d.to_string()),
                        _ => Err(SpannerDbErr::TypeConversion {
                            column: param_name.to_string(),
                            expected: "Decimal".to_string(),
                            got: format!("element [{}]: {:?}", i, v),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
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
        fields: &[Field],
        idx: usize,
        column_name: &str,
    ) -> Result<Value, DbErr> {
        let type_code = fields
            .get(idx)
            .and_then(|f| f.r#type.as_ref())
            .map(|t| t.code)
            .ok_or_else(|| {
                DbErr::Type(format!(
                    "Missing type metadata for column {} at index {}",
                    column_name, idx
                ))
            })?;

        match TypeCode::try_from(type_code) {
            Ok(TypeCode::Bool) => {
                let v = row.column::<Option<bool>>(idx).map_err(|e| {
                    DbErr::Type(format!(
                        "Failed to read BOOL column {} at index {}: {}",
                        column_name, idx, e
                    ))
                })?;
                Ok(Value::Bool(v))
            }
            Ok(TypeCode::Int64) => {
                let v = row.column::<Option<i64>>(idx).map_err(|e| {
                    DbErr::Type(format!(
                        "Failed to read INT64 column {} at index {}: {}",
                        column_name, idx, e
                    ))
                })?;
                Ok(Value::BigInt(v))
            }
            Ok(TypeCode::Float64 | TypeCode::Float32) => {
                let v = row.column::<Option<f64>>(idx).map_err(|e| {
                    DbErr::Type(format!(
                        "Failed to read FLOAT64 column {} at index {}: {}",
                        column_name, idx, e
                    ))
                })?;
                Ok(Value::Double(v))
            }
            Ok(TypeCode::String) => {
                let v = row.column::<Option<String>>(idx).map_err(|e| {
                    DbErr::Type(format!(
                        "Failed to read STRING column {} at index {}: {}",
                        column_name, idx, e
                    ))
                })?;
                Ok(Value::String(v))
            }
            Ok(TypeCode::Bytes) => {
                let v = row.column::<Option<Vec<u8>>>(idx).map_err(|e| {
                    DbErr::Type(format!(
                        "Failed to read BYTES column {} at index {}: {}",
                        column_name, idx, e
                    ))
                })?;
                Ok(Value::Bytes(v))
            }
            Ok(TypeCode::Timestamp) => {
                #[cfg(feature = "with-chrono")]
                {
                    let v = row
                        .column::<Option<time::OffsetDateTime>>(idx)
                        .map_err(|e| {
                            DbErr::Type(format!(
                                "Failed to read TIMESTAMP column {} at index {}: {}",
                                column_name, idx, e
                            ))
                        })?;
                    match v {
                        Some(odt) => {
                            let chrono_dt = chrono::DateTime::from_timestamp(
                                odt.unix_timestamp(),
                                odt.nanosecond(),
                            )
                            .unwrap_or(chrono::DateTime::UNIX_EPOCH);
                            Ok(Value::ChronoDateTime(Some(chrono_dt.naive_utc())))
                        }
                        None => Ok(Value::ChronoDateTime(None)),
                    }
                }
                #[cfg(not(feature = "with-chrono"))]
                {
                    let v = row.column::<Option<String>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read TIMESTAMP column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    Ok(Value::String(v))
                }
            }
            Ok(TypeCode::Date) => {
                #[cfg(feature = "with-chrono")]
                {
                    let v = row.column::<Option<time::Date>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read DATE column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    match v {
                        Some(d) => {
                            let naive_date = chrono::NaiveDate::from_ymd_opt(
                                d.year(),
                                d.month() as u32,
                                d.day() as u32,
                            );
                            Ok(Value::ChronoDate(naive_date))
                        }
                        None => Ok(Value::ChronoDate(None)),
                    }
                }
                #[cfg(not(feature = "with-chrono"))]
                {
                    let v = row.column::<Option<String>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read DATE column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    Ok(Value::String(v))
                }
            }
            Ok(TypeCode::Numeric) => {
                #[cfg(feature = "with-rust_decimal")]
                {
                    let big_decimal = row
                        .column::<Option<gcloud_spanner::bigdecimal::BigDecimal>>(idx)
                        .map_err(|e| {
                            DbErr::Type(format!(
                                "Failed to read NUMERIC column {} at index {}: {}",
                                column_name, idx, e
                            ))
                        })?;
                    match big_decimal {
                        Some(bd) => {
                            let decimal = rust_decimal::Decimal::from_str_exact(&bd.to_string())
                                .map_err(|e| {
                                    DbErr::Type(format!(
                                        "Failed to convert NUMERIC column {} to Decimal: {}",
                                        column_name, e
                                    ))
                                })?;
                            Ok(Value::Decimal(Some(decimal)))
                        }
                        None => Ok(Value::Decimal(None)),
                    }
                }
                #[cfg(not(feature = "with-rust_decimal"))]
                {
                    let v = row.column::<Option<String>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read NUMERIC column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    Ok(Value::String(v))
                }
            }
            Ok(TypeCode::Json) => {
                #[cfg(feature = "with-json")]
                {
                    let v = row.column::<Option<String>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read JSON column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    match v {
                        Some(s) => {
                            let json =
                                serde_json::from_str::<serde_json::Value>(&s).map_err(|e| {
                                    DbErr::Type(format!(
                                        "Failed to parse JSON column {}: {}",
                                        column_name, e
                                    ))
                                })?;
                            Ok(Value::Json(Some(Box::new(json))))
                        }
                        None => Ok(Value::Json(None)),
                    }
                }
                #[cfg(not(feature = "with-json"))]
                {
                    let v = row.column::<Option<String>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read JSON column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    Ok(Value::String(v))
                }
            }
            Ok(TypeCode::Uuid) => {
                #[cfg(feature = "with-uuid")]
                {
                    let v = row.column::<Option<String>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read UUID column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    match v {
                        Some(s) => {
                            let uuid = uuid::Uuid::parse_str(&s).map_err(|e| {
                                DbErr::Type(format!(
                                    "UUID column {} at index {} contains invalid value '{}': {}",
                                    column_name, idx, s, e
                                ))
                            })?;
                            Ok(Value::Uuid(Some(uuid)))
                        }
                        None => Ok(Value::Uuid(None)),
                    }
                }
                #[cfg(not(feature = "with-uuid"))]
                {
                    let v = row.column::<Option<String>>(idx).map_err(|e| {
                        DbErr::Type(format!(
                            "Failed to read UUID column {} at index {}: {}",
                            column_name, idx, e
                        ))
                    })?;
                    Ok(Value::String(v))
                }
            }
            Ok(TypeCode::Array) => Self::read_array_value(row, fields, idx, column_name),
            _ => Err(DbErr::Type(format!(
                "Unsupported type code {} for column {} at index {}",
                type_code, column_name, idx
            ))),
        }
    }

    fn read_array_value(
        row: &gcloud_spanner::row::Row,
        fields: &[Field],
        idx: usize,
        column_name: &str,
    ) -> Result<Value, DbErr> {
        let element_type_code = fields
            .get(idx)
            .and_then(|f| f.r#type.as_ref())
            .and_then(|t| t.array_element_type.as_ref())
            .map(|et| et.code)
            .unwrap_or(0);

        let err = |e: gcloud_spanner::row::Error| -> DbErr {
            DbErr::Type(format!(
                "Failed to read ARRAY column {} at index {}: {}",
                column_name, idx, e
            ))
        };

        match TypeCode::try_from(element_type_code) {
            Ok(TypeCode::Bool) => {
                let arr = row.column::<Option<Vec<bool>>>(idx).map_err(err)?;
                match arr {
                    Some(v) => {
                        let values: Vec<Value> =
                            v.into_iter().map(|v| Value::Bool(Some(v))).collect();
                        Ok(Value::Array(ArrayType::Bool, Some(Box::new(values))))
                    }
                    None => Ok(Value::Array(ArrayType::Bool, None)),
                }
            }
            Ok(TypeCode::Int64) => {
                let arr = row.column::<Option<Vec<i64>>>(idx).map_err(err)?;
                match arr {
                    Some(v) => {
                        let values: Vec<Value> =
                            v.into_iter().map(|v| Value::BigInt(Some(v))).collect();
                        Ok(Value::Array(ArrayType::BigInt, Some(Box::new(values))))
                    }
                    None => Ok(Value::Array(ArrayType::BigInt, None)),
                }
            }
            Ok(TypeCode::Float64 | TypeCode::Float32) => {
                let arr = row.column::<Option<Vec<f64>>>(idx).map_err(err)?;
                match arr {
                    Some(v) => {
                        let values: Vec<Value> =
                            v.into_iter().map(|v| Value::Double(Some(v))).collect();
                        Ok(Value::Array(ArrayType::Double, Some(Box::new(values))))
                    }
                    None => Ok(Value::Array(ArrayType::Double, None)),
                }
            }
            Ok(TypeCode::String) => {
                let arr = row.column::<Option<Vec<String>>>(idx).map_err(err)?;
                match arr {
                    Some(v) => {
                        let values: Vec<Value> =
                            v.into_iter().map(|v| Value::String(Some(v))).collect();
                        Ok(Value::Array(ArrayType::String, Some(Box::new(values))))
                    }
                    None => Ok(Value::Array(ArrayType::String, None)),
                }
            }
            Ok(TypeCode::Bytes) => {
                let arr = row.column::<Option<Vec<Vec<u8>>>>(idx).map_err(err)?;
                match arr {
                    Some(v) => {
                        let values: Vec<Value> =
                            v.into_iter().map(|v| Value::Bytes(Some(v))).collect();
                        Ok(Value::Array(ArrayType::Bytes, Some(Box::new(values))))
                    }
                    None => Ok(Value::Array(ArrayType::Bytes, None)),
                }
            }
            _ => Err(DbErr::Type(format!(
                "Unsupported array element type {} for column {}",
                element_type_code, column_name
            ))),
        }
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
                            || (!bytes[next_idx].is_ascii_alphanumeric() && bytes[next_idx] != b'_')
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
        let mut fields: Option<std::sync::Arc<Vec<Field>>> = None;

        while let Some(row) = iter
            .next()
            .await
            .map_err(|e| SpannerDbErr::Query(e.to_string()))?
        {
            if column_names.is_none() {
                column_names = Some(Self::extract_column_names_from_statement(&statement));
                fields = Some(iter.columns_metadata().clone());
            }

            let col_names = column_names.as_ref().unwrap();
            let col_fields = fields.as_ref().unwrap();
            let mut values = BTreeMap::new();

            for (idx, col_name) in col_names.iter().enumerate() {
                let value =
                    Self::spanner_value_to_sea_value(&row, col_fields.as_ref(), idx, col_name)?;
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
        tracing::warn!(
            "SpannerProxy::begin() is a no-op. Spanner uses callback-based transactions via \
             SpannerConnection::read_write_transaction() instead of begin/commit/rollback."
        );
    }

    async fn commit(&self) {
        tracing::warn!(
            "SpannerProxy::commit() is a no-op. Spanner transactions are committed \
             automatically when the callback passed to read_write_transaction() succeeds."
        );
    }

    async fn rollback(&self) {
        tracing::warn!(
            "SpannerProxy::rollback() is a no-op. Spanner transactions are rolled back \
             automatically when the callback passed to read_write_transaction() returns an error."
        );
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
