use super::error::ErrorCode;
use super::types::{
    CredentialDefinition, CredentialRequestMetadata, RevocationRegistryDefinition,
    W3CCredential,
};

use anoncreds::Error;
use std::sync::Arc;

use anoncreds::types::{
    LinkSecret,
};

pub struct W3cProcess {}

impl W3cProcess {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl W3cProcess {
    pub fn process_credential(
        &self,
        cred: Arc<W3CCredential>,
        cred_req_metadata: Arc<CredentialRequestMetadata>,
        link_secret: String,
        cred_def: Arc<CredentialDefinition>,
        rev_reg_def: Option<Arc<RevocationRegistryDefinition>>,
    ) -> Result<Arc<W3CCredential>, ErrorCode> {

        let link_secret =
            LinkSecret::try_from(link_secret.as_str()).map_err(|err| Error::from(err))?;
        let rev_reg_def = rev_reg_def.as_ref().map(|def| &def.0);
        let mut new_cred = cred.0.clone(); //.try_clone().map_err(|err| Error::from(err))?;
        
        anoncreds::w3c::prover::process_credential(
            &mut new_cred,
            &cred_req_metadata.0,
            &link_secret,
            &cred_def.0,
            rev_reg_def,
        )?;

        Ok(Arc::new(W3CCredential(new_cred)))
        //Ok(serde_json::to_string(&new_cred)?)
    
    }
}