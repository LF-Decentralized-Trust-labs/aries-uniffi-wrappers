use aries_askar::{Error, ErrorKind};
use aries_askar::storage::{Error as StorageError, ErrorKind as StorageErrorKind};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone, Serialize, uniffi::Error, thiserror::Error)]
pub enum ErrorCode {
    Backend { error_message: String },
    Busy { error_message: String },
    Duplicate { error_message: String },
    Encryption { error_message: String },
    Input { error_message: String },
    NotFound { error_message: String },
    Unexpected { error_message: String },
    Unsupported { error_message: String },
    Custom { error_message: String },
}

impl From<Error> for ErrorCode {
    fn from(err: Error) -> ErrorCode {
        match err.kind() {
            ErrorKind::Backend => ErrorCode::Backend {
                error_message: err.to_string(),
            },
            ErrorKind::Busy => ErrorCode::Busy {
                error_message: err.to_string(),
            },
            ErrorKind::Duplicate => ErrorCode::Duplicate {
                error_message: err.to_string(),
            },
            ErrorKind::Encryption => ErrorCode::Encryption {
                error_message: err.to_string(),
            },
            ErrorKind::Input => ErrorCode::Input {
                error_message: err.to_string(),
            },
            ErrorKind::NotFound => ErrorCode::NotFound {
                error_message: err.to_string(),
            },
            ErrorKind::Unexpected => ErrorCode::Unexpected {
                error_message: err.to_string(),
            },
            ErrorKind::Unsupported => ErrorCode::Unsupported {
                error_message: err.to_string(),
            },
            ErrorKind::Custom => ErrorCode::Custom {
                error_message: err.to_string(),
            },
        }
    }
}

impl From<StorageError> for ErrorCode {
    fn from(err: StorageError) -> ErrorCode {
        match err.kind() {
            StorageErrorKind::Backend => ErrorCode::Backend {
                error_message: err.to_string(),
            },
            StorageErrorKind::Busy => ErrorCode::Busy {
                error_message: err.to_string(),
            },
            StorageErrorKind::Duplicate => ErrorCode::Duplicate {
                error_message: err.to_string(),
            },
            StorageErrorKind::Encryption => ErrorCode::Encryption {
                error_message: err.to_string(),
            },
            StorageErrorKind::Input => ErrorCode::Input {
                error_message: err.to_string(),
            },
            StorageErrorKind::NotFound => ErrorCode::NotFound {
                error_message: err.to_string(),
            },
            StorageErrorKind::Unexpected => ErrorCode::Unexpected {
                error_message: err.to_string(),
            },
            StorageErrorKind::Unsupported => ErrorCode::Unsupported {
                error_message: err.to_string(),
            },
            StorageErrorKind::Custom => ErrorCode::Custom {
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
