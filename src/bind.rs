use crate::error::SpannerDbErr;
use gcloud_spanner::statement::Statement as SpannerStatement;
use sea_orm::{DbErr, Statement};

/// Convert a SeaORM Statement into a Spanner Statement with bound parameters.
pub(crate) fn convert_statement(stmt: &Statement) -> Result<SpannerStatement, DbErr> {
    let sql = &stmt.sql;
    let mut spanner_stmt = SpannerStatement::new(sql);

    if let Some(values) = &stmt.values {
        for (idx, value) in values.0.iter().enumerate() {
            let param_name = format!("p{}", idx + 1);
            bind_value(&mut spanner_stmt, &param_name, value)?;
        }
    }

    Ok(spanner_stmt)
}

/// Bind a sea_orm::Value to a Spanner Statement parameter.
///
/// This is the shared implementation used by SpannerExecutor, SpannerReadWriteTransaction,
/// and SpannerReadOnlyTransaction. The proxy path (SpannerProxy) has its own implementation
/// that uses custom wrapper types for Spanner-specific type handling.
pub(crate) fn bind_value(
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
        Value::String(Some(v)) => {
            let s: &str = v.as_ref();
            stmt.add_param(param_name, &s)
        }
        Value::String(None) => stmt.add_param(param_name, &Option::<String>::None),
        Value::Char(Some(v)) => stmt.add_param(param_name, &v.to_string()),
        Value::Char(None) => stmt.add_param(param_name, &Option::<String>::None),
        Value::Bytes(Some(v)) => stmt.add_param(param_name, v.as_ref()),
        Value::Bytes(None) => stmt.add_param(param_name, &Option::<Vec<u8>>::None),

        #[cfg(feature = "with-chrono")]
        Value::ChronoDate(Some(v)) => stmt.add_param(param_name, &v.format("%Y-%m-%d").to_string()),
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
            &crate::chrono_support::SpannerTimestamp::new(v.and_utc()),
        ),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTime(None) => stmt.add_param(
            param_name,
            &crate::chrono_support::SpannerOptionalTimestamp::none(),
        ),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeUtc(Some(v)) => stmt.add_param(
            param_name,
            &crate::chrono_support::SpannerTimestamp::new(v.to_utc()),
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
        Value::Uuid(Some(v)) => stmt.add_param(param_name, &v.to_string()),
        #[cfg(feature = "with-uuid")]
        Value::Uuid(None) => stmt.add_param(param_name, &Option::<String>::None),

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
