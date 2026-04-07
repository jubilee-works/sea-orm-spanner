use {
    gcloud_googleapis::spanner::v1::TypeCode,
    gcloud_spanner::statement::{single_type, ToKind},
    prost_types::value::Kind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpannerUuid(pub uuid::Uuid);

impl SpannerUuid {
    pub fn new(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }

    pub fn inner(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl From<uuid::Uuid> for SpannerUuid {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

impl From<SpannerUuid> for uuid::Uuid {
    fn from(wrapper: SpannerUuid) -> Self {
        wrapper.0
    }
}

impl ToKind for SpannerUuid {
    fn to_kind(&self) -> Kind {
        Kind::StringValue(self.0.hyphenated().to_string())
    }

    fn get_type() -> gcloud_googleapis::spanner::v1::Type {
        single_type(TypeCode::Uuid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpannerOptionalUuid(pub Option<uuid::Uuid>);

impl SpannerOptionalUuid {
    pub fn some(uuid: uuid::Uuid) -> Self {
        Self(Some(uuid))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl From<Option<uuid::Uuid>> for SpannerOptionalUuid {
    fn from(uuid: Option<uuid::Uuid>) -> Self {
        Self(uuid)
    }
}

impl ToKind for SpannerOptionalUuid {
    fn to_kind(&self) -> Kind {
        match &self.0 {
            Some(v) => Kind::StringValue(v.hyphenated().to_string()),
            None => Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> gcloud_googleapis::spanner::v1::Type {
        single_type(TypeCode::Uuid)
    }
}
