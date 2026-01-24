use gcloud_gax::grpc::Status;
use gcloud_gax::retry::TryAs;
use gcloud_spanner::session::SessionError;
use sea_orm::DbErr;
use thiserror::Error;

/// Error type for use in Spanner transactions that satisfies google-cloud-spanner bounds
#[derive(Debug)]
pub enum SpannerTxError {
    Grpc(Status),
    Session(SessionError),
    Db(DbErr),
}

impl From<Status> for SpannerTxError {
    fn from(err: Status) -> Self {
        SpannerTxError::Grpc(err)
    }
}

impl From<SessionError> for SpannerTxError {
    fn from(err: SessionError) -> Self {
        SpannerTxError::Session(err)
    }
}

impl From<DbErr> for SpannerTxError {
    fn from(err: DbErr) -> Self {
        SpannerTxError::Db(err)
    }
}

impl TryAs<Status> for SpannerTxError {
    fn try_as(&self) -> Option<&Status> {
        match self {
            SpannerTxError::Grpc(s) => Some(s),
            _ => None,
        }
    }
}

impl std::fmt::Display for SpannerTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpannerTxError::Grpc(e) => write!(f, "gRPC error: {}", e),
            SpannerTxError::Session(e) => write!(f, "Session error: {}", e),
            SpannerTxError::Db(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for SpannerTxError {}

#[derive(Error, Debug)]
pub enum SpannerDbErr {
    #[error("Spanner connection error: {0}")]
    Connection(String),

    #[error("Spanner query error: {0}")]
    Query(String),

    #[error("Spanner execution error: {0}")]
    Execution(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Row parse error: {0}")]
    RowParse(String),

    #[error("Type conversion error: column={column}, expected={expected}, got={got}")]
    TypeConversion {
        column: String,
        expected: String,
        got: String,
    },

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<SpannerDbErr> for DbErr {
    fn from(err: SpannerDbErr) -> Self {
        match err {
            SpannerDbErr::Connection(msg) => DbErr::Conn(sea_orm::RuntimeErr::Internal(msg)),
            SpannerDbErr::Query(msg) => DbErr::Query(sea_orm::RuntimeErr::Internal(msg)),
            SpannerDbErr::Execution(msg) => DbErr::Exec(sea_orm::RuntimeErr::Internal(msg)),
            SpannerDbErr::Transaction(msg) => DbErr::Query(sea_orm::RuntimeErr::Internal(msg)),
            SpannerDbErr::RowParse(msg) => DbErr::Type(msg),
            SpannerDbErr::TypeConversion {
                column,
                expected,
                got,
            } => DbErr::Type(format!(
                "column={}, expected={}, got={}",
                column, expected, got
            )),
            SpannerDbErr::ColumnNotFound(col) => DbErr::Type(format!("column not found: {}", col)),
            SpannerDbErr::InvalidConfig(msg) => DbErr::Conn(sea_orm::RuntimeErr::Internal(msg)),
        }
    }
}

impl From<gcloud_spanner::client::Error> for SpannerDbErr {
    fn from(err: gcloud_spanner::client::Error) -> Self {
        SpannerDbErr::Connection(err.to_string())
    }
}

impl From<gcloud_spanner::row::Error> for SpannerDbErr {
    fn from(err: gcloud_spanner::row::Error) -> Self {
        SpannerDbErr::RowParse(err.to_string())
    }
}
