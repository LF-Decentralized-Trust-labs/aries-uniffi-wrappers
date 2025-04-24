use super::error::ErrorCode;
use super::types::{
    Credential, CredentialDefinition, CredentialDefinitionPrivate, CredentialDefinitionTuple,
    CredentialKeyCorrectnessProof, CredentialOffer, CredentialRequest, CredentialRevocationConfig,
    RevocationRegistryDefinition, RevocationRegistryDefinitionPrivate,
    RevocationRegistryDefinitionTuple, RevocationStatusList, Schema,
};
use anoncreds::data_types::{cred_def::SignatureType, rev_reg_def::RegistryType};
use anoncreds::issuer::{
    create_credential, create_credential_definition, create_credential_offer,
    create_revocation_registry_def, create_revocation_status_list, create_schema,
    update_revocation_status_list,
};
use anoncreds::tails::TailsFileWriter;
use anoncreds::types::{CredentialDefinitionConfig, MakeCredentialValues};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Issuer {}

impl Issuer {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl Issuer {
    pub fn create_schema(
        &self,
        schema_name: String,
        schema_version: String,
        issuer_id: String,
        attr_names: Vec<String>,
    ) -> Result<Arc<Schema>, ErrorCode> {
        let schema = create_schema(
            schema_name.as_str(),
            schema_version.as_str(),
            issuer_id
                .try_into()
                .map_err(|err| anoncreds::Error::from(err))?,
            attr_names.into(),
        )?;
        Ok(Arc::new(Schema(schema)))
    }

    pub fn create_credential_definition(
        &self,
        schema_id: String,
        schema: Arc<Schema>,
        tag: String,
        issuer_id: String,
        support_revocation: bool,
    ) -> Result<CredentialDefinitionTuple, ErrorCode> {
        let (cred_def, cred_def_priv, key_proof) = create_credential_definition(
            schema_id
                .try_into()
                .map_err(|err| anoncreds::Error::from(err))?,
            &schema.0,
            issuer_id
                .try_into()
                .map_err(|err| anoncreds::Error::from(err))?,
            tag.as_str(),
            SignatureType::CL,
            CredentialDefinitionConfig { support_revocation },
        )?;
        Ok(CredentialDefinitionTuple {
            cred_def: Arc::new(CredentialDefinition(cred_def)),
            cred_def_priv: Arc::new(CredentialDefinitionPrivate(cred_def_priv)),
            key_correctness_proof: Arc::new(CredentialKeyCorrectnessProof(key_proof)),
        })
    }

    pub fn create_credential_offer(
        &self,
        schema_id: String,
        cred_def_id: String,
        key_proof: Arc<CredentialKeyCorrectnessProof>,
    ) -> Result<Arc<CredentialOffer>, ErrorCode> {
        let cred_offer = create_credential_offer(
            schema_id
                .try_into()
                .map_err(|err| anoncreds::Error::from(err))?,
            cred_def_id
                .try_into()
                .map_err(|err| anoncreds::Error::from(err))?,
            &key_proof.0,
        )?;
        Ok(Arc::new(CredentialOffer(cred_offer)))
    }

    pub fn create_revocation_registry_def(
        &self,
        cred_def: Arc<CredentialDefinition>,
        cred_def_id: String,
        tag: String,
        max_cred_num: u32,
        tails_dir_path: Option<String>,
    ) -> Result<RevocationRegistryDefinitionTuple, ErrorCode> {
        let mut tails_writer = TailsFileWriter::new(tails_dir_path);
        let (rev_reg_def, rev_reg_def_private) = create_revocation_registry_def(
            &cred_def.0,
            cred_def_id
                .try_into()
                .map_err(|err| anoncreds::Error::from(err))?,
            tag.as_str(),
            RegistryType::CL_ACCUM,
            max_cred_num,
            &mut tails_writer,
        )?;
        Ok(RevocationRegistryDefinitionTuple {
            rev_reg_def: Arc::new(RevocationRegistryDefinition(rev_reg_def)),
            rev_reg_def_priv: Arc::new(RevocationRegistryDefinitionPrivate(rev_reg_def_private)),
        })
    }

    pub fn create_revocation_status_list(
        &self,
        cred_def: Arc<CredentialDefinition>,
        rev_reg_def_id: String,
        rev_reg_def: Arc<RevocationRegistryDefinition>,
        rev_reg_priv: Arc<RevocationRegistryDefinitionPrivate>,
        timestamp: Option<u64>,
        issuance_by_default: bool,
    ) -> Result<Arc<RevocationStatusList>, ErrorCode> {
        let rev_status_list = create_revocation_status_list(
            &cred_def.0,
            rev_reg_def_id
                .try_into()
                .map_err(|err| anoncreds::Error::from(err))?,
            &rev_reg_def.0,
            &rev_reg_priv.0,
            issuance_by_default,
            timestamp,
        )?;
        Ok(Arc::new(RevocationStatusList(rev_status_list)))
    }

    pub fn update_revocation_status_list(
        &self,
        cred_def: Arc<CredentialDefinition>,
        timestamp: Option<u64>,
        issued: Option<Vec<u32>>,
        revoked: Option<Vec<u32>>,
        rev_reg_def: Arc<RevocationRegistryDefinition>,
        rev_reg_priv: Arc<RevocationRegistryDefinitionPrivate>,
        current_list: Arc<RevocationStatusList>,
    ) -> Result<Arc<RevocationStatusList>, ErrorCode> {
        let issued: Option<BTreeSet<u32>> = issued.map(|v| v.into_iter().collect());
        let revoked: Option<BTreeSet<u32>> = revoked.map(|v| v.into_iter().collect());
        let rev_status_list = update_revocation_status_list(
            &cred_def.0,
            &rev_reg_def.0,
            &rev_reg_priv.0,
            &current_list.0,
            issued,
            revoked,
            timestamp,
        )?;
        Ok(Arc::new(RevocationStatusList(rev_status_list)))
    }

    pub fn create_credential(
        &self,
        cred_def: Arc<CredentialDefinition>,
        cred_def_private: Arc<CredentialDefinitionPrivate>,
        cred_offer: Arc<CredentialOffer>,
        cred_request: Arc<CredentialRequest>,
        attr_raw_values: HashMap<String, String>,
        attr_enc_values: Option<HashMap<String, String>>,
        revocation_config: Option<CredentialRevocationConfig>,
    ) -> Result<Arc<Credential>, ErrorCode> {
        if attr_raw_values.is_empty() {
            return Err(ErrorCode::Input {
                error_message: "Cannot create credential with no attribute".to_string(),
            });
        }

        let mut cred_values = MakeCredentialValues::default();
        for (name, raw) in attr_raw_values.iter() {
            let encoded = attr_enc_values
                .as_ref()
                .and_then(|attr_enc_values| attr_enc_values.get(name));
            if let Some(encoded) = encoded {
                cred_values.add_encoded(name, raw, encoded.clone());
            } else {
                cred_values.add_raw(name, raw)?;
            }
        }

        let cred = create_credential(
            &cred_def.0,
            &cred_def_private.0,
            &cred_offer.0,
            &cred_request.0,
            cred_values.into(),
            revocation_config.as_ref().map(|config| config.into()),
        )?;
        Ok(Arc::new(Credential(cred)))
    }
}
