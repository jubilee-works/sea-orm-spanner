use {
    gcloud_googleapis::spanner::v1::TypeCode,
    gcloud_spanner::statement::{single_type, ToKind},
    prost_types::value::Kind,
};

pub struct SpannerJson(pub serde_json::Value);

impl SpannerJson {
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl ToKind for SpannerJson {
    fn to_kind(&self) -> Kind {
        Kind::StringValue(self.0.to_string())
    }

    fn get_type() -> gcloud_googleapis::spanner::v1::Type {
        single_type(TypeCode::Json)
    }
}

pub struct SpannerOptionalJson(pub Option<serde_json::Value>);

impl SpannerOptionalJson {
    pub fn some(value: serde_json::Value) -> Self {
        Self(Some(value))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalJson {
    fn to_kind(&self) -> Kind {
        match &self.0 {
            Some(v) => Kind::StringValue(v.to_string()),
            None => Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> gcloud_googleapis::spanner::v1::Type {
        single_type(TypeCode::Json)
    }
}
