use anoncreds::{Error, ErrorKind};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone, uniffi::Error, thiserror::Error)]
pub enum ErrorCode {
    Input { error_message: String },
    IOError { error_message: String },
    InvalidState { error_message: String },
    Unexpected { error_message: String },
    CredentialRevoked { error_message: String },
    InvalidUserRevocId { error_message: String },
    ProofRejected { error_message: String },
    RevocationRegistryFull { error_message: String },
}

impl From<Error> for ErrorCode {
    fn from(err: Error) -> ErrorCode {
        match err.kind() {
            ErrorKind::Input => ErrorCode::Input {
                error_message: err.to_string(),
            },
            ErrorKind::IOError => ErrorCode::IOError {
                error_message: err.to_string(),
            },
            ErrorKind::InvalidState => ErrorCode::InvalidState {
                error_message: err.to_string(),
            },
            ErrorKind::Unexpected => ErrorCode::Unexpected {
                error_message: err.to_string(),
            },
            ErrorKind::CredentialRevoked => ErrorCode::CredentialRevoked {
                error_message: err.to_string(),
            },
            ErrorKind::InvalidUserRevocId => ErrorCode::InvalidUserRevocId {
                error_message: err.to_string(),
            },
            ErrorKind::ProofRejected => ErrorCode::ProofRejected {
                error_message: err.to_string(),
            },
            ErrorKind::RevocationRegistryFull => ErrorCode::RevocationRegistryFull {
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

impl From<serde_json::Error> for ErrorCode {
    fn from(err: serde_json::Error) -> Self {
        ErrorCode::Input {
            error_message: err.to_string(),
        }
    }
}
