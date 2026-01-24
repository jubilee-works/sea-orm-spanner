use google_cloud_googleapis::spanner::v1::{Type, TypeAnnotationCode, TypeCode};
use google_cloud_spanner::statement::ToKind;
use prost_types::{value, ListValue, Value};

pub struct SpannerInt64Array(pub Vec<i64>);

impl ToKind for SpannerInt64Array {
    fn to_kind(&self) -> value::Kind {
        value::Kind::ListValue(ListValue {
            values: self
                .0
                .iter()
                .map(|x| Value {
                    kind: Some(x.to_string().to_kind()),
                })
                .collect(),
        })
    }

    fn get_type() -> Type {
        Type {
            code: TypeCode::Array.into(),
            array_element_type: Some(Box::new(Type {
                code: TypeCode::Int64.into(),
                array_element_type: None,
                struct_type: None,
                type_annotation: TypeAnnotationCode::Unspecified.into(),
                proto_type_fqn: String::new(),
            })),
            struct_type: None,
            type_annotation: TypeAnnotationCode::Unspecified.into(),
            proto_type_fqn: String::new(),
        }
    }
}

pub struct SpannerOptionalInt64Array(pub Option<Vec<i64>>);

impl SpannerOptionalInt64Array {
    pub fn some(v: Vec<i64>) -> Self {
        Self(Some(v))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalInt64Array {
    fn to_kind(&self) -> value::Kind {
        match &self.0 {
            Some(v) => SpannerInt64Array(v.clone()).to_kind(),
            None => value::Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> Type {
        SpannerInt64Array::get_type()
    }
}

pub struct SpannerFloat64Array(pub Vec<f64>);

impl ToKind for SpannerFloat64Array {
    fn to_kind(&self) -> value::Kind {
        value::Kind::ListValue(ListValue {
            values: self
                .0
                .iter()
                .map(|x| Value {
                    kind: Some(value::Kind::NumberValue(*x)),
                })
                .collect(),
        })
    }

    fn get_type() -> Type {
        Type {
            code: TypeCode::Array.into(),
            array_element_type: Some(Box::new(Type {
                code: TypeCode::Float64.into(),
                array_element_type: None,
                struct_type: None,
                type_annotation: TypeAnnotationCode::Unspecified.into(),
                proto_type_fqn: String::new(),
            })),
            struct_type: None,
            type_annotation: TypeAnnotationCode::Unspecified.into(),
            proto_type_fqn: String::new(),
        }
    }
}

pub struct SpannerOptionalFloat64Array(pub Option<Vec<f64>>);

impl SpannerOptionalFloat64Array {
    pub fn some(v: Vec<f64>) -> Self {
        Self(Some(v))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalFloat64Array {
    fn to_kind(&self) -> value::Kind {
        match &self.0 {
            Some(v) => SpannerFloat64Array(v.clone()).to_kind(),
            None => value::Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> Type {
        SpannerFloat64Array::get_type()
    }
}

pub struct SpannerStringArray(pub Vec<String>);

impl ToKind for SpannerStringArray {
    fn to_kind(&self) -> value::Kind {
        value::Kind::ListValue(ListValue {
            values: self
                .0
                .iter()
                .map(|x| Value {
                    kind: Some(value::Kind::StringValue(x.clone())),
                })
                .collect(),
        })
    }

    fn get_type() -> Type {
        Type {
            code: TypeCode::Array.into(),
            array_element_type: Some(Box::new(Type {
                code: TypeCode::String.into(),
                array_element_type: None,
                struct_type: None,
                type_annotation: TypeAnnotationCode::Unspecified.into(),
                proto_type_fqn: String::new(),
            })),
            struct_type: None,
            type_annotation: TypeAnnotationCode::Unspecified.into(),
            proto_type_fqn: String::new(),
        }
    }
}

pub struct SpannerOptionalStringArray(pub Option<Vec<String>>);

impl SpannerOptionalStringArray {
    pub fn some(v: Vec<String>) -> Self {
        Self(Some(v))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalStringArray {
    fn to_kind(&self) -> value::Kind {
        match &self.0 {
            Some(v) => SpannerStringArray(v.clone()).to_kind(),
            None => value::Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> Type {
        SpannerStringArray::get_type()
    }
}

pub struct SpannerBoolArray(pub Vec<bool>);

impl ToKind for SpannerBoolArray {
    fn to_kind(&self) -> value::Kind {
        value::Kind::ListValue(ListValue {
            values: self
                .0
                .iter()
                .map(|x| Value {
                    kind: Some(value::Kind::BoolValue(*x)),
                })
                .collect(),
        })
    }

    fn get_type() -> Type {
        Type {
            code: TypeCode::Array.into(),
            array_element_type: Some(Box::new(Type {
                code: TypeCode::Bool.into(),
                array_element_type: None,
                struct_type: None,
                type_annotation: TypeAnnotationCode::Unspecified.into(),
                proto_type_fqn: String::new(),
            })),
            struct_type: None,
            type_annotation: TypeAnnotationCode::Unspecified.into(),
            proto_type_fqn: String::new(),
        }
    }
}

pub struct SpannerOptionalBoolArray(pub Option<Vec<bool>>);

impl SpannerOptionalBoolArray {
    pub fn some(v: Vec<bool>) -> Self {
        Self(Some(v))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalBoolArray {
    fn to_kind(&self) -> value::Kind {
        match &self.0 {
            Some(v) => SpannerBoolArray(v.clone()).to_kind(),
            None => value::Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> Type {
        SpannerBoolArray::get_type()
    }
}

pub struct SpannerBytesArray(pub Vec<Vec<u8>>);

impl ToKind for SpannerBytesArray {
    fn to_kind(&self) -> value::Kind {
        value::Kind::ListValue(ListValue {
            values: self
                .0
                .iter()
                .map(|x| Value {
                    kind: Some(value::Kind::StringValue(base64::encode(x))),
                })
                .collect(),
        })
    }

    fn get_type() -> Type {
        Type {
            code: TypeCode::Array.into(),
            array_element_type: Some(Box::new(Type {
                code: TypeCode::Bytes.into(),
                array_element_type: None,
                struct_type: None,
                type_annotation: TypeAnnotationCode::Unspecified.into(),
                proto_type_fqn: String::new(),
            })),
            struct_type: None,
            type_annotation: TypeAnnotationCode::Unspecified.into(),
            proto_type_fqn: String::new(),
        }
    }
}

pub struct SpannerOptionalBytesArray(pub Option<Vec<Vec<u8>>>);

impl SpannerOptionalBytesArray {
    pub fn some(v: Vec<Vec<u8>>) -> Self {
        Self(Some(v))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

impl ToKind for SpannerOptionalBytesArray {
    fn to_kind(&self) -> value::Kind {
        match &self.0 {
            Some(v) => SpannerBytesArray(v.clone()).to_kind(),
            None => value::Kind::NullValue(prost_types::NullValue::NullValue.into()),
        }
    }

    fn get_type() -> Type {
        SpannerBytesArray::get_type()
    }
}
