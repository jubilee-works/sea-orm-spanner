use sea_query::Value;

pub fn value_to_spanner_literal(value: &Value) -> String {
    match value {
        Value::Bool(Some(b)) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        Value::Bool(None) => "NULL".to_string(),
        Value::TinyInt(Some(i)) => i.to_string(),
        Value::TinyInt(None) => "NULL".to_string(),
        Value::SmallInt(Some(i)) => i.to_string(),
        Value::SmallInt(None) => "NULL".to_string(),
        Value::Int(Some(i)) => i.to_string(),
        Value::Int(None) => "NULL".to_string(),
        Value::BigInt(Some(i)) => i.to_string(),
        Value::BigInt(None) => "NULL".to_string(),
        Value::TinyUnsigned(Some(i)) => i.to_string(),
        Value::TinyUnsigned(None) => "NULL".to_string(),
        Value::SmallUnsigned(Some(i)) => i.to_string(),
        Value::SmallUnsigned(None) => "NULL".to_string(),
        Value::Unsigned(Some(i)) => i.to_string(),
        Value::Unsigned(None) => "NULL".to_string(),
        Value::BigUnsigned(Some(i)) => i.to_string(),
        Value::BigUnsigned(None) => "NULL".to_string(),
        Value::Float(Some(f)) => format!("{:.}", f),
        Value::Float(None) => "NULL".to_string(),
        Value::Double(Some(d)) => format!("{:.}", d),
        Value::Double(None) => "NULL".to_string(),
        Value::String(Some(s)) => escape_string(s),
        Value::String(None) => "NULL".to_string(),
        Value::Char(Some(c)) => escape_string(&c.to_string()),
        Value::Char(None) => "NULL".to_string(),
        Value::Bytes(Some(b)) => format!("B\"{}\"", bytes_to_hex(b)),
        Value::Bytes(None) => "NULL".to_string(),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDate(Some(d)) => format!("DATE '{}'", d.format("%Y-%m-%d")),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDate(None) => "NULL".to_string(),
        #[cfg(feature = "with-chrono")]
        Value::ChronoTime(Some(t)) => format!("'{}'", t.format("%H:%M:%S%.f")),
        #[cfg(feature = "with-chrono")]
        Value::ChronoTime(None) => "NULL".to_string(),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTime(Some(dt)) => {
            format!("TIMESTAMP '{}'", dt.format("%Y-%m-%dT%H:%M:%S%.fZ"))
        }
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTime(None) => "NULL".to_string(),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeUtc(Some(dt)) => {
            format!("TIMESTAMP '{}'", dt.format("%Y-%m-%dT%H:%M:%S%.fZ"))
        }
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeUtc(None) => "NULL".to_string(),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeLocal(Some(dt)) => {
            format!("TIMESTAMP '{}'", dt.format("%Y-%m-%dT%H:%M:%S%.fZ"))
        }
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeLocal(None) => "NULL".to_string(),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeWithTimeZone(Some(dt)) => {
            format!("TIMESTAMP '{}'", dt.format("%Y-%m-%dT%H:%M:%S%.fZ"))
        }
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeWithTimeZone(None) => "NULL".to_string(),
        #[cfg(feature = "with-uuid")]
        Value::Uuid(Some(u)) => format!("'{}'", u.hyphenated().to_string()),
        #[cfg(feature = "with-uuid")]
        Value::Uuid(None) => "NULL".to_string(),
        #[cfg(feature = "with-json")]
        Value::Json(Some(j)) => escape_string(&j.to_string()),
        #[cfg(feature = "with-json")]
        Value::Json(None) => "NULL".to_string(),
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(Some(d)) => d.to_string(),
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(None) => "NULL".to_string(),
        #[allow(unreachable_patterns)]
        _ => "NULL".to_string(),
    }
}

fn escape_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''").replace('\\', "\\\\"))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
