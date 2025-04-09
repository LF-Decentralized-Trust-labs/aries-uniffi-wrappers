use std::sync::Arc;

use anoncreds::data_types::issuer_id::IssuerId;
use anoncreds::data_types::w3c::VerifiableCredentialSpecVersion;
use anoncreds::w3c::credential_conversion::credential_from_w3c;
use anoncreds::{
    data_types::w3c::credential::W3CCredential, w3c::credential_conversion::credential_to_w3c,
};
use std::convert::TryFrom;
use super::{error::ErrorCode, types::Credential};

pub struct CredentialConversions {}

impl CredentialConversions {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl CredentialConversions {
    pub fn credential_from_w3c_json(
        &self,
        w3c_credential_json: String,
    ) -> Result<Arc<Credential>, ErrorCode> {
        let w3c_credential = serde_json::from_str::<W3CCredential>(&w3c_credential_json)?;
        let rust_cred = credential_from_w3c(&w3c_credential)?;

        Ok(Arc::new(Credential(rust_cred)))
    }

    pub fn credential_to_w3c_json(
        &self,
        credential: Arc<Credential>,
        issuer_id_string: String,
        version_string: Option<String>,
    ) -> Result<String, ErrorCode> {
        let issuer_id = IssuerId::new(&issuer_id_string).expect("Error initializing issuer_id");
        let version = match version_string {
            Some(ref v) => Some(VerifiableCredentialSpecVersion::try_from(v.as_str())?),
            None => None
        };

        let w3c_credential: W3CCredential = credential_to_w3c(&credential.0, &issuer_id, version)?;

        Ok(serde_json::to_string(&w3c_credential)?)
    }
}
