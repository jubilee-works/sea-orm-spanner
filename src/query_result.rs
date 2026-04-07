use {
    gcloud_spanner::row::Row as SpannerRow,
    sea_orm::{DbErr, TryGetError},
};

pub struct SpannerQueryResult {
    row: SpannerRow,
}

impl SpannerQueryResult {
    pub fn new(row: SpannerRow) -> Self {
        Self { row }
    }

    pub fn try_get_by_name<T>(&self, col: &str) -> Result<T, TryGetError>
    where
        T: SpannerTryGet,
    {
        T::try_get_spanner_by_name(&self.row, col)
    }

    pub fn try_get_by_index<T>(&self, idx: usize) -> Result<T, TryGetError>
    where
        T: SpannerTryGet,
    {
        T::try_get_spanner(&self.row, idx)
    }
}

pub trait SpannerTryGet: Sized {
    fn try_get_spanner(row: &SpannerRow, idx: usize) -> Result<Self, TryGetError>;
    fn try_get_spanner_by_name(row: &SpannerRow, name: &str) -> Result<Self, TryGetError>;
}

macro_rules! impl_spanner_try_get {
    ($ty:ty) => {
        impl SpannerTryGet for $ty {
            fn try_get_spanner(row: &SpannerRow, idx: usize) -> Result<Self, TryGetError> {
                row.column::<$ty>(idx)
                    .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
            }
            fn try_get_spanner_by_name(row: &SpannerRow, name: &str) -> Result<Self, TryGetError> {
                row.column_by_name::<$ty>(name)
                    .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
            }
        }
    };
    ($ty:ty, $spanner_ty:ty) => {
        impl SpannerTryGet for $ty {
            fn try_get_spanner(row: &SpannerRow, idx: usize) -> Result<Self, TryGetError> {
                row.column::<$spanner_ty>(idx)
                    .map(|v| v as $ty)
                    .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
            }
            fn try_get_spanner_by_name(row: &SpannerRow, name: &str) -> Result<Self, TryGetError> {
                row.column_by_name::<$spanner_ty>(name)
                    .map(|v| v as $ty)
                    .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
            }
        }
    };
}

impl_spanner_try_get!(String);
impl_spanner_try_get!(i64);
impl_spanner_try_get!(i32, i64);
impl_spanner_try_get!(i16, i64);
impl_spanner_try_get!(i8, i64);
impl_spanner_try_get!(f64);
impl_spanner_try_get!(f32, f64);
impl_spanner_try_get!(bool);
impl_spanner_try_get!(Vec<u8>);

impl<T: SpannerTryGet> SpannerTryGet for Option<T> {
    fn try_get_spanner(row: &SpannerRow, idx: usize) -> Result<Self, TryGetError> {
        match T::try_get_spanner(row, idx) {
            Ok(v) => Ok(Some(v)),
            Err(TryGetError::Null(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
    fn try_get_spanner_by_name(row: &SpannerRow, name: &str) -> Result<Self, TryGetError> {
        match T::try_get_spanner_by_name(row, name) {
            Ok(v) => Ok(Some(v)),
            Err(TryGetError::Null(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
