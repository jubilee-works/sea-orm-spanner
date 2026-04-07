use {
    gcloud_googleapis::spanner::v1::{Type, TypeCode},
    gcloud_spanner::statement::{single_type, ToKind},
    prost_types::value::Kind,
};

pub struct SpannerTimestamp(pub chrono::DateTime<chrono::Utc>);

impl SpannerTimestamp {
    pub fn new(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt)
    }
}

impl ToKind for SpannerTimestamp {
    fn to_kind(&self) -> Kind {
        Kind::StringValue(self.0.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
    }

    fn get_type() -> Type {
        single_type(TypeCode::Timestamp)
    }
}

pub struct SpannerOptionalTimestamp(pub Option<chrono::DateTime<chrono::Utc>>);

impl SpannerOptionalTimestamp {
    pub fn some(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(Some(dt))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalTimestamp {
    fn to_kind(&self) -> Kind {
        match &self.0 {
            Some(dt) => SpannerTimestamp(*dt).to_kind(),
            None => Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> Type {
        single_type(TypeCode::Timestamp)
    }
}

pub struct SpannerNaiveDateTime(pub chrono::NaiveDateTime);

impl SpannerNaiveDateTime {
    pub fn new(dt: chrono::NaiveDateTime) -> Self {
        Self(dt)
    }
}

impl ToKind for SpannerNaiveDateTime {
    fn to_kind(&self) -> Kind {
        let utc = self.0.and_utc();
        Kind::StringValue(utc.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
    }

    fn get_type() -> Type {
        single_type(TypeCode::Timestamp)
    }
}

pub struct SpannerOptionalNaiveDateTime(pub Option<chrono::NaiveDateTime>);

impl SpannerOptionalNaiveDateTime {
    pub fn some(dt: chrono::NaiveDateTime) -> Self {
        Self(Some(dt))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalNaiveDateTime {
    fn to_kind(&self) -> Kind {
        match &self.0 {
            Some(dt) => SpannerNaiveDateTime(*dt).to_kind(),
            None => Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> Type {
        single_type(TypeCode::Timestamp)
    }
}
