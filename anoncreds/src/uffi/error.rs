use anoncreds::{Error, ErrorKind};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone, uniffi::Error, thiserror::Error)]
pub enum ErrorCode {
    Input { msg: String },
    IOError { msg: String },
    InvalidState { msg: String },
    Unexpected { msg: String },
    CredentialRevoked { msg: String },
    InvalidUserRevocId { msg: String },
    ProofRejected { msg: String },
    RevocationRegistryFull { msg: String },
}

impl From<Error> for ErrorCode {
    fn from(err: Error) -> ErrorCode {
        match err.kind() {
            ErrorKind::Input => ErrorCode::Input {
                msg: err.to_string(),
            },
            ErrorKind::IOError => ErrorCode::IOError {
                msg: err.to_string(),
            },
            ErrorKind::InvalidState => ErrorCode::InvalidState {
                msg: err.to_string(),
            },
            ErrorKind::Unexpected => ErrorCode::Unexpected {
                msg: err.to_string(),
            },
            ErrorKind::CredentialRevoked => ErrorCode::CredentialRevoked {
                msg: err.to_string(),
            },
            ErrorKind::InvalidUserRevocId => ErrorCode::InvalidUserRevocId {
                msg: err.to_string(),
            },
            ErrorKind::ProofRejected => ErrorCode::ProofRejected {
                msg: err.to_string(),
            },
            ErrorKind::RevocationRegistryFull => ErrorCode::RevocationRegistryFull {
                msg: err.to_string(),
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
            msg: err.to_string(),
        }
    }
}
