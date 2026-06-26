mod uffi;
use uffi::credential_conversions::CredentialConversions;
use uffi::issuer::Issuer;
use uffi::prover::Prover;
use uffi::types::{
    Credential, CredentialDefinition, CredentialDefinitionPrivate, CredentialKeyCorrectnessProof,
    CredentialOffer, CredentialRequest, CredentialRequestMetadata, CredentialRevocationState,
    Presentation, PresentationRequest, RevocationRegistryDefinition,
    RevocationRegistryDefinitionPrivate, RevocationRegistryDelta, RevocationStatusList, Schema,
    W3CCredential,
};
use uffi::verifier::Verifier;
use uffi::w3c::W3cProcess;

uniffi::include_scaffolding!("anoncreds_uniffi");
