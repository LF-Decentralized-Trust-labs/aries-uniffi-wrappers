use super::error::ErrorCode;
use anoncreds::data_types::{
    cred_def::CredentialDefinition as RustCredentialDefinition,
    rev_status_list::RevocationStatusList as RustRevocationStatusList,
    schema::Schema as RustSchema,
};
use anoncreds::types::{
    Credential as RustCredential, CredentialDefinitionPrivate as RustCredentialDefinitionPrivate,
    CredentialKeyCorrectnessProof as RustCredentialKeyCorrectnessProof,
    CredentialOffer as RustCredentialOffer, CredentialRequest as RustCredentialRequest,
    CredentialRequestMetadata as RustCredentialRequestMetadata,
    CredentialRevocationConfig as RustCredentialRevocationConfig,
    CredentialRevocationState as RustCredentialRevocationState, Presentation as RustPresentation,
    PresentationRequest as RustPresentationRequest,
    RevocationRegistryDefinition as RustRevocationRegistryDefinition,
    RevocationRegistryDefinitionPrivate as RustRevocationRegistryDefinitionPrivate,
};
use anoncreds::data_types::w3c::credential::W3CCredential as RustW3cCredential;
use anoncreds_clsignatures::RevocationRegistryDelta as RustRevocationRegistryDelta;
use std::collections::HashMap;
use std::sync::Arc;
use anoncreds::data_types::w3c::context::Context;



pub struct Schema(pub RustSchema);

#[uniffi::export]
impl Schema {
    #[uniffi::constructor]
    pub fn new(json: String) -> Result<Arc<Self>, ErrorCode> {
        Ok(Arc::new(Self(serde_json::from_str::<RustSchema>(&json)?)))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }

    pub fn schema_id(&self) -> String {
        format!(
            "{}:2:{}:{}",
            self.0.issuer_id.0, self.0.name, self.0.version
        )
    }

    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    pub fn version(&self) -> String {
        self.0.version.clone()
    }

    pub fn issuer_id(&self) -> String {
        self.0.issuer_id.0.clone()
    }
}

pub struct CredentialDefinition(pub RustCredentialDefinition);

#[uniffi::export]
impl CredentialDefinition {
    #[uniffi::constructor]
    pub fn new(json: String) -> Result<Arc<Self>, ErrorCode> {
        Ok(Arc::new(Self(serde_json::from_str::<
            RustCredentialDefinition,
        >(&json)?)))
    }

    pub fn schema_id(&self) -> String {
        self.0.schema_id.0.clone()
    }

    pub fn cred_def_id(&self) -> String {
        format!(
            "{}:3:CL:{}:{}",
            self.0.issuer_id.0,
            self.schema_id(),
            self.0.tag
        )
    }

    pub fn issuer_id(&self) -> String {
        self.0.issuer_id.0.clone()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }
}

#[derive(uniffi::Record)]
pub struct CredentialDefinitionTuple {
    pub cred_def: Arc<CredentialDefinition>,
    pub cred_def_priv: Arc<CredentialDefinitionPrivate>,
    pub key_correctness_proof: Arc<CredentialKeyCorrectnessProof>,
}

pub struct CredentialOffer(pub RustCredentialOffer);

#[uniffi::export]
impl CredentialOffer {
    #[uniffi::constructor]
    pub fn new(json: String) -> Result<Arc<Self>, ErrorCode> {
        Ok(Arc::new(Self(serde_json::from_str::<RustCredentialOffer>(
            &json,
        )?)))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }

    pub fn cred_def_id(&self) -> String {
        self.0.cred_def_id.0.clone()
    }
}

#[derive(uniffi::Record)]
pub struct CredentialRequestTuple {
    pub request: Arc<CredentialRequest>,
    pub metadata: Arc<CredentialRequestMetadata>,
}

#[derive(Debug)]
pub struct RevocationRegistryDefinition(pub RustRevocationRegistryDefinition);

#[uniffi::export]
impl RevocationRegistryDefinition {
    #[uniffi::constructor]
    pub fn new(json: String) -> Result<Arc<Self>, ErrorCode> {
        Ok(Arc::new(Self(serde_json::from_str::<
            RustRevocationRegistryDefinition,
        >(&json)?)))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }

    pub fn max_cred_num(&self) -> u32 {
        self.0.value.max_cred_num
    }

    pub fn tails_hash(&self) -> String {
        self.0.value.tails_hash.clone()
    }

    pub fn tails_location(&self) -> String {
        self.0.value.tails_location.clone()
    }

    pub fn rev_reg_id(&self) -> String {
        format!(
            "{}:4:{}:CL_ACCUM:{}",
            self.0.issuer_id.0, self.0.cred_def_id.0, self.0.tag
        )
    }

    pub fn issuer_id(&self) -> String {
        self.0.issuer_id.0.clone()
    }
}

#[derive(uniffi::Record)]
pub struct RevocationRegistryDefinitionTuple {
    pub rev_reg_def: Arc<RevocationRegistryDefinition>,
    pub rev_reg_def_priv: Arc<RevocationRegistryDefinitionPrivate>,
}

#[derive(uniffi::Record)]
pub struct CredentialRevocationConfig {
    pub reg_def: Arc<RevocationRegistryDefinition>,
    pub reg_def_private: Arc<RevocationRegistryDefinitionPrivate>,
    pub status_list: Arc<RevocationStatusList>,
    pub registry_index: u32,
}

impl<'a> From<&'a CredentialRevocationConfig> for RustCredentialRevocationConfig<'a> {
    fn from(revocation_config: &'a CredentialRevocationConfig) -> Self {
        RustCredentialRevocationConfig {
            reg_def: &revocation_config.reg_def.0,
            reg_def_private: &revocation_config.reg_def_private.0,
            status_list: &revocation_config.status_list.0,
            registry_idx: revocation_config.registry_index,
        }
    }
}

#[derive(Debug)]
pub struct Credential(pub RustCredential);

#[uniffi::export]
impl Credential {
    #[uniffi::constructor]
    pub fn new(json: String) -> Result<Arc<Self>, ErrorCode> {
        Ok(Arc::new(Self(serde_json::from_str::<RustCredential>(
            &json,
        )?)))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }

    pub fn schema_id(&self) -> String {
        self.0.schema_id.0.clone()
    }

    pub fn cred_def_id(&self) -> String {
        self.0.cred_def_id.0.clone()
    }

    pub fn rev_reg_id(&self) -> Option<String> {
        self.0.rev_reg_id.as_ref().map(|id| id.0.clone())
    }

    pub fn rev_reg_index(&self) -> Option<u32> {
        self.0.signature.extract_index()
    }

    pub fn values(&self) -> HashMap<String, String> {
        self.0
            .values
            .0
            .iter()
            .map(|(key, value)| (key.clone(), value.raw.clone()))
            .collect()
    }
}

#[derive(Debug)]
pub struct W3CCredential(pub RustW3cCredential);

#[uniffi::export]
impl W3CCredential {
    
    #[uniffi::constructor]
    pub fn new(json: String) -> Result<Arc<Self>, ErrorCode> {
        println!("new w3c credential constructor");

        match serde_json::from_str::<RustW3cCredential>(&json) {
            Ok(parsed_cred) => {
                println!("Credential created: {:?}", parsed_cred);
                Ok(Arc::new(Self(parsed_cred)))
            }
            Err(e) => {
                println!("Error creating credential: {:?}", e);
                Err(e.into()) // assuming your ErrorCode implements From<serde_json::Error>
            }
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }

    pub fn context(&self) -> String {
        self.0.context.0
        .iter()
        .map(|ctx| match ctx {
            Context::URI(uri) => uri.0.clone(),
            Context::Object(obj) => obj.to_string(),
        })
        .collect()
    }

    pub fn id(&self) -> Option<String> {
        self.0.id.as_ref().map(|uri| uri.0.clone())
    }
    
    pub fn r#type(&self) -> Vec<String> {
        self.0.type_.0.iter().cloned().collect()
    }

    pub fn issuer(&self) -> String {
        serde_json::to_string(&self.0.issuer).unwrap()
    }

    pub fn credential_subject(&self) -> String {
        serde_json::to_string(&self.0.credential_subject).unwrap()
    }

    pub fn issuance_date(&self) -> String {
        serde_json::to_string(&self.0.issuance_date).unwrap()
    }

    pub fn proof(&self) -> String {
        serde_json::to_string(&self.0.proof).unwrap()
    }

}

#[derive(uniffi::Record)]
pub struct RequestedCredential {
    pub cred: Arc<Credential>,
    pub timestamp: Option<u64>,
    pub rev_state: Option<Arc<CredentialRevocationState>>,
    pub requested_attributes: HashMap<String, bool>,
    pub requested_predicates: Vec<String>,
}

macro_rules! define_serializable_struct {
    ($struct_name:ident, $rust_struct_name:ident) => {
        pub struct $struct_name(pub $rust_struct_name);

        #[uniffi::export]
        impl $struct_name {
            #[uniffi::constructor]
            pub fn new(json: String) -> Result<Arc<Self>, ErrorCode> {
                Ok(Arc::new(Self(serde_json::from_str::<$rust_struct_name>(
                    &json,
                )?)))
            }

            pub fn to_json(&self) -> String {
                serde_json::to_string(&self.0).unwrap()
            }
        }
    };
}

define_serializable_struct!(CredentialDefinitionPrivate, RustCredentialDefinitionPrivate);
define_serializable_struct!(
    CredentialKeyCorrectnessProof,
    RustCredentialKeyCorrectnessProof
);
define_serializable_struct!(CredentialRequest, RustCredentialRequest);
define_serializable_struct!(CredentialRequestMetadata, RustCredentialRequestMetadata);
define_serializable_struct!(RevocationRegistryDelta, RustRevocationRegistryDelta);
define_serializable_struct!(CredentialRevocationState, RustCredentialRevocationState);
define_serializable_struct!(RevocationStatusList, RustRevocationStatusList);
define_serializable_struct!(PresentationRequest, RustPresentationRequest);
define_serializable_struct!(Presentation, RustPresentation);
define_serializable_struct!(
    RevocationRegistryDefinitionPrivate,
    RustRevocationRegistryDefinitionPrivate
);
