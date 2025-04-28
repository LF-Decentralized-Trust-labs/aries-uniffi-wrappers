use indy_vdr::common::error::{VdrError, VdrErrorKind};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone, uniffi::Error, thiserror::Error)]
pub enum ErrorCode {
    Config { error_message: String },
    Connection { error_message: String },
    FileSystem { error_message: String },
    Input { error_message: String },
    Resource { error_message: String },
    Unavailable { error_message: String },
    Unexpected { error_message: String },
    Incompatible { error_message: String },
    PoolNoConsensus { error_message: String },
    PoolRequestFailed { error_message: String },
    PoolTimeout { error_message: String },
    Resolver { error_message: String },
    Success {},
}

impl From<VdrError> for ErrorCode {
    fn from(err: VdrError) -> ErrorCode {
        match err.kind() {
            VdrErrorKind::Config => ErrorCode::Config {
                error_message: err.to_string(),
            },
            VdrErrorKind::Connection => ErrorCode::Connection {
                error_message: err.to_string(),
            },
            VdrErrorKind::FileSystem => ErrorCode::FileSystem {
                error_message: err.to_string(),
            },
            VdrErrorKind::Input => ErrorCode::Input {
                error_message: err.to_string(),
            },
            VdrErrorKind::Resource => ErrorCode::Resource {
                error_message: err.to_string(),
            },
            VdrErrorKind::Unavailable => ErrorCode::Unavailable {
                error_message: err.to_string(),
            },
            VdrErrorKind::Unexpected => ErrorCode::Unexpected {
                error_message: err.to_string(),
            },
            VdrErrorKind::Incompatible => ErrorCode::Incompatible {
                error_message: err.to_string(),
            },
            VdrErrorKind::PoolNoConsensus => ErrorCode::PoolNoConsensus {
                error_message: err.to_string(),
            },
            VdrErrorKind::PoolRequestFailed(_) => ErrorCode::PoolRequestFailed {
                error_message: err.to_string(),
            },
            VdrErrorKind::PoolTimeout => ErrorCode::PoolTimeout {
                error_message: err.to_string(),
            },
            VdrErrorKind::Resolver => ErrorCode::Resolver {
                error_message: err.to_string(),
            },
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn input_err<M>(msg: M) -> ErrorCode
where
    M: fmt::Display + Send + Sync + 'static,
{
    ErrorCode::Input {
        error_message: msg.to_string(),
    }
}
