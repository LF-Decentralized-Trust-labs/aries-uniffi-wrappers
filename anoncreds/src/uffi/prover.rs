use super::error::ErrorCode;
use super::types::{
    Credential, CredentialDefinition, CredentialOffer, CredentialRequest,
    CredentialRequestMetadata, CredentialRequestTuple, CredentialRevocationState, Presentation,
    PresentationRequest, RequestedCredential, RevocationRegistryDefinition,
    RevocationRegistryDelta, RevocationStatusList, Schema,
};
use anoncreds::data_types::{cred_def::CredentialDefinitionId, schema::SchemaId};
use anoncreds::prover::{
    create_credential_request, create_or_update_revocation_state, create_presentation,
};
use anoncreds::tails::TailsFileReader;
use anoncreds::types::{
    CredentialRevocationState as RustCredentialRevocationState, LinkSecret, PresentCredentials,
};
use anoncreds::Error;
use anoncreds_clsignatures::{RevocationRegistry, Witness};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Prover {}

impl Prover {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl Prover {
    pub fn create_credential_request(
        &self,
        entropy: Option<String>,
        prover_did: Option<String>,
        cred_def: Arc<CredentialDefinition>,
        link_secret: String,
        link_secret_id: String,
        cred_offer: Arc<CredentialOffer>,
    ) -> Result<CredentialRequestTuple, ErrorCode> {
        let link_secret =
            LinkSecret::try_from(link_secret.as_str()).map_err(|err| Error::from(err))?;
        let (cred_req, cred_req_metadata) = create_credential_request(
            entropy.as_deref(),
            prover_did.as_deref(),
            &cred_def.0,
            &link_secret,
            link_secret_id.as_str(),
            &cred_offer.0,
        )?;
        Ok(CredentialRequestTuple {
            request: Arc::new(CredentialRequest(cred_req)),
            metadata: Arc::new(CredentialRequestMetadata(cred_req_metadata)),
        })
    }

    pub fn process_credential(
        &self,
        cred: Arc<Credential>,
        cred_req_metadata: Arc<CredentialRequestMetadata>,
        link_secret: String,
        cred_def: Arc<CredentialDefinition>,
        rev_reg_def: Option<Arc<RevocationRegistryDefinition>>,
    ) -> Result<Arc<Credential>, ErrorCode> {
        let link_secret =
            LinkSecret::try_from(link_secret.as_str()).map_err(|err| Error::from(err))?;
        let rev_reg_def = rev_reg_def.as_ref().map(|def| &def.0);
        let mut new_cred = cred.0.try_clone().map_err(|err| Error::from(err))?;
        anoncreds::prover::process_credential(
            &mut new_cred,
            &cred_req_metadata.0,
            &link_secret,
            &cred_def.0,
            rev_reg_def,
        )?;
        Ok(Arc::new(Credential(new_cred)))
    }

    pub fn create_presentation(
        &self,
        pres_req: Arc<PresentationRequest>,
        requested_credentials: Vec<RequestedCredential>,
        self_attested_attributes: Option<HashMap<String, String>>,
        link_secret: String,
        schemas: HashMap<String, Arc<Schema>>,
        cred_defs: HashMap<String, Arc<CredentialDefinition>>,
    ) -> Result<Arc<Presentation>, ErrorCode> {
        let link_secret =
            LinkSecret::try_from(link_secret.as_str()).map_err(|err| Error::from(err))?;

        let mut present_creds = PresentCredentials::default();
        for rc in &requested_credentials {
            let mut add_cred = present_creds.add_credential(
                &rc.cred.0,
                rc.timestamp,
                rc.rev_state.as_ref().map(|s| &s.0),
            );

            for (referent, revealed) in &rc.requested_attributes {
                add_cred.add_requested_attribute(referent, *revealed);
            }
            for referent in &rc.requested_predicates {
                add_cred.add_requested_predicate(referent);
            }
        }

        let mut schemas_with_id = HashMap::new();
        for (id, schema) in schemas.into_iter() {
            let schema_id = SchemaId::new(id).map_err(|err| Error::from(err))?;
            schemas_with_id.insert(schema_id, schema.0.clone());
        }

        let mut cred_defs_with_id = HashMap::new();
        for (id, cred_def) in cred_defs.into_iter() {
            let cred_def_id = CredentialDefinitionId::new(id).map_err(|err| Error::from(err))?;
            let cred_def = cred_def.0.try_clone().map_err(|err| Error::from(err))?;
            cred_defs_with_id.insert(cred_def_id, cred_def);
        }

        let presentation = create_presentation(
            &pres_req.0,
            present_creds,
            self_attested_attributes,
            &link_secret,
            &schemas_with_id,
            &cred_defs_with_id,
        )?;
        Ok(Arc::new(Presentation(presentation)))
    }

    pub fn create_or_update_revocation_state(
        &self,
        rev_reg_def: Arc<RevocationRegistryDefinition>,
        rev_status_list: Arc<RevocationStatusList>,
        rev_reg_idx: u32,
        tails_path: String,
        rev_state: Option<Arc<CredentialRevocationState>>,
        old_rev_status_list: Option<Arc<RevocationStatusList>>,
    ) -> Result<Arc<CredentialRevocationState>, ErrorCode> {
        let rev_state = create_or_update_revocation_state(
            &tails_path,
            &rev_reg_def.0,
            &rev_status_list.0,
            rev_reg_idx,
            rev_state.as_ref().map(|s| &s.0),
            old_rev_status_list.as_ref().map(|s| &s.0),
        )?;
        Ok(Arc::new(CredentialRevocationState(rev_state)))
    }

    pub fn create_revocation_state(
        &self,
        rev_reg_def: Arc<RevocationRegistryDefinition>,
        rev_reg_delta: Arc<RevocationRegistryDelta>,
        timestamp: u64,
        rev_reg_idx: u32,
        tails_path: String,
    ) -> Result<Arc<CredentialRevocationState>, ErrorCode> {
        let tails_reader = TailsFileReader::new(&tails_path)?;
        let witness = Witness::new(
            rev_reg_idx,
            rev_reg_def.max_cred_num(),
            // issuance by default
            true,
            &rev_reg_delta.0,
            &tails_reader,
        )
        .map_err(|err| ErrorCode::Input {
            error_message: format!("Witness error: {}", err),
        })?;
        let rev_reg = RevocationRegistry::from(rev_reg_delta.0.clone());
        Ok(Arc::new(CredentialRevocationState(
            RustCredentialRevocationState {
                witness,
                rev_reg,
                timestamp,
            },
        )))
    }
}
