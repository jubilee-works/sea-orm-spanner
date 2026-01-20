use sea_query::ColumnType;

pub fn spanner_type_name(col_type: &ColumnType) -> String {
    match col_type {
        ColumnType::Char(_) | ColumnType::String(_) | ColumnType::Text => "STRING(MAX)".to_string(),
        ColumnType::TinyInteger | ColumnType::SmallInteger | ColumnType::Integer | ColumnType::BigInteger => {
            "INT64".to_string()
        }
        ColumnType::TinyUnsigned | ColumnType::SmallUnsigned | ColumnType::Unsigned | ColumnType::BigUnsigned => {
            "INT64".to_string()
        }
        ColumnType::Float => "FLOAT32".to_string(),
        ColumnType::Double => "FLOAT64".to_string(),
        ColumnType::Decimal(_) | ColumnType::Money(_) => "NUMERIC".to_string(),
        ColumnType::DateTime => "TIMESTAMP".to_string(),
        ColumnType::Timestamp => "TIMESTAMP".to_string(),
        ColumnType::TimestampWithTimeZone => "TIMESTAMP".to_string(),
        ColumnType::Time => "STRING(MAX)".to_string(),
        ColumnType::Date => "DATE".to_string(),
        ColumnType::Year => "INT64".to_string(),
        ColumnType::Interval(_, _) => "INT64".to_string(),
        ColumnType::Binary(_) | ColumnType::VarBinary(_) | ColumnType::Blob => "BYTES(MAX)".to_string(),
        ColumnType::Bit(_) => "BYTES(MAX)".to_string(),
        ColumnType::Boolean => "BOOL".to_string(),
        ColumnType::Json | ColumnType::JsonBinary => "JSON".to_string(),
        ColumnType::Uuid => "STRING(36)".to_string(),
        ColumnType::Array(inner) => format!("ARRAY<{}>", spanner_type_name(inner)),
        ColumnType::Cidr | ColumnType::Inet | ColumnType::MacAddr => "STRING(MAX)".to_string(),
        ColumnType::LTree => "STRING(MAX)".to_string(),
        ColumnType::Enum { name: _, .. } => "STRING(MAX)".to_string(),
        ColumnType::Custom(name) => name.to_string().to_uppercase(),
        _ => "STRING(MAX)".to_string(),
    }
}

pub fn spanner_column_def(col_type: &ColumnType, is_nullable: bool) -> String {
    let type_name = spanner_type_name(col_type);
    if is_nullable {
        type_name
    } else {
        format!("{} NOT NULL", type_name)
    }
}
