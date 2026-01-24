//! UUID support for Google Cloud Spanner native UUID type.
//!
//! This module provides a wrapper type and ToKind implementation
//! for using Spanner's native UUID type instead of STRING(36).

use gcloud_googleapis::spanner::v1::{Type, TypeAnnotationCode};
use gcloud_spanner::statement::ToKind;
use prost_types::value::Kind;

const TYPE_CODE_UUID: i32 = 17;

/// Wrapper type for uuid::Uuid that implements ToKind for Spanner's native UUID type.
///
/// Spanner's UUID type is encoded as a lowercase hexadecimal string
/// as described in RFC 9562, section 4.
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

    fn get_type() -> Type
    where
        Self: Sized,
    {
        Type {
            code: TYPE_CODE_UUID,
            array_element_type: None,
            struct_type: None,
            type_annotation: TypeAnnotationCode::Unspecified.into(),
            proto_type_fqn: String::new(),
        }
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

    fn get_type() -> Type
    where
        Self: Sized,
    {
        SpannerUuid::get_type()
    }
}
