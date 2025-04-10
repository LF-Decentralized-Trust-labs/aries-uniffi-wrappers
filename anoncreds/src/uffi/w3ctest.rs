//use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::string::ToString;

use anoncreds::data_types::w3c::context::Contexts;
use anoncreds::data_types::w3c::credential_attributes::CredentialSubject;
use anoncreds::data_types::w3c::proof::{
    CredentialPresentationProofValue, CredentialSignatureProofValue, DataIntegrityProof,
};
use anoncreds::data_types::w3c::VerifiableCredentialSpecVersion;
use anoncreds::data_types::{
    issuer_id::IssuerId,
    w3c::{constants::W3C_CREDENTIAL_TYPE, one_or_many::OneOrMany, uri::URI},
};



// pub const W3C_CREDENTIAL_TYPE: &str = "VerifiableCredential";
// pub const W3C_PRESENTATION_TYPE: &str = "VerifiablePresentation";

// pub(crate) static ANONCREDS_CREDENTIAL_TYPES: Lazy<Types> =
//     Lazy::new(|| Types(HashSet::from([String::from(W3C_CREDENTIAL_TYPE)])));

// pub(crate) static ANONCREDS_PRESENTATION_TYPES: Lazy<Types> =
//     Lazy::new(|| Types(HashSet::from([String::from(W3C_PRESENTATION_TYPE)])));


#[cfg(test)]
mod tests {
    //use anoncreds::data_types::w3c::credential::W3CCredential;

    // #[test]
    // fn serde_w3c_credential() {
    //     let cred_json = include_str!("sample_credential.json");
    //     let cred1: W3CCredential =
    //         serde_json::from_str(&cred_json).expect("Error deserializing w3c credential");
    //     let out_json = serde_json::to_string(&cred1).expect("Error serializing w3c credential");
    //     let cred2: W3CCredential =
    //         serde_json::from_str(&out_json).expect("Error deserializing w3c credential");
    //     assert_eq!(cred1, cred2);
    // }


    use std::any::Any;
    use anoncreds::data_types::nonce::Nonce;
    use anoncreds::types::CredentialRequestMetadata;
    use anoncreds_clsignatures::bn::BigNumber;
    use anoncreds_clsignatures::CredentialSecretsBlindingFactors;
    use crate::uffi::types::{Credential, RevocationRegistryDefinition};
    use crate::uffi::w3c::W3cProcess;
    use super::super::types::W3CCredential;

    #[test]
    fn teste_w3c_1() {
        let cred_json = include_str!("sample_credential.json");
        let credential = W3CCredential::new(cred_json.to_string());
        //println!("credential: {:?}", credential);

        assert_eq!("a", "a");
    }

    #[test]
    fn teste_w3c_attributes() {
        let cred_json = include_str!("sample_credential.json");

        let credential = W3CCredential::new(cred_json.to_string()).unwrap();
        println!("w3c credential: {:?}\n", credential);


        let context = credential.context();
        println!("Context: {:?}\n", context);

        assert!(context.contains("https://www.w3.org/ns/credentials"));

        let issuer = credential.issuer();
        println!("Issuer: {}\n", issuer);

        let id = credential.id();
        println!("Id: {:?}\n", id);

        let types = credential.r#type();
        println!("Type: {:?}\n", types);

        let credential_subject = credential.credential_subject();
        println!("Credential Subject: {:?}\n", credential_subject);

        let issueance_date = credential.issuance_date();
        println!("Issueance Date: {:?}\n", issueance_date);
    }

    // #[test]
    // fn teste_w3c_cred() {
    //     let anoncreds_cred_json = include_str!("anoncreds_credential.json");
    //     let cred_json = include_str!("sample_credential.json");
    //
    //     let parsed: serde_json::Value = serde_json::from_str(anoncreds_cred_json).unwrap();
    //     println!("Credential parsed: {:?}\n", parsed);
    //
    //     let credentiallegacy = Credential::new(anoncreds_cred_json.to_string());
    //     assert!(credentiallegacy.is_ok(), "Erro ao criar credential: {:?}", credentiallegacy);
    //     let credentiallegacy = credentiallegacy.unwrap();
    //     println!("Credential parsed: {:?}\n", credentiallegacy);
    //
    //     let credential = W3CCredential::new(cred_json.to_string()).unwrap();
    //     println!("Credential w3c: {:?}\n", credential);
    //
    //     let rev_reg_def  = include_str!("rev_reg.json");
    //     let rev_reg_def_obj = RevocationRegistryDefinition::new(rev_reg_def.to_string());
    //     assert!(rev_reg_def_obj.is_ok(), "Erro ao criar rev_reg: {:?}", rev_reg_def_obj);
    //     let rev_reg_def_obj_parsed = rev_reg_def_obj.unwrap();
    //     println!("rev reg parsed: {:?}\n", rev_reg_def_obj_parsed);
    //
    //     let credential_request_metadata  = include_str!("credential_request_metadata.json");
    //
    //
    //     let vr_prime = None;
    //     let v_prime = BigNumber::from_dec("26229531560803327955462519947982737473128331437268046125026825867717092835641742168097583674615660306385828655760652000784511092542288483191315439767609670762243249770474773152944058864549187821980345572106977549072191097067007516379039295546634629639527163762468962903855912262649223635340496302367223743439598774350284814170290432221103856675179707611975601521787767701648558117369789834002923089984983875934244152960046125784484975825805601288695619950601416918604462431826515597463219371469020984311177916581543976244764002039873150555593206430312208444175994530953778234221887856099782864288099390415162022718920616606210403491530469948").unwrap();
    //     // Monta a struct
    //     let blinding_factors = CredentialSecretsBlindingFactors {
    //         v_prime,
    //         vr_prime,
    //     };
    //
    //     let nonce = Nonce::new().unwrap();
    //
    //     //{"link_secret_blinding_data":{"v_prime":,"vr_prime":"041F701B842B4BEA741D8938F4792014AA6E910F59D8990FD38D4C8DF552FBF5"},"nonce":"834155596455072625638932","link_secret_name":"9717533e-d110-476f-915b-ab3d63ac694b"}
    //     let metadata = CredentialRequestMetadata {
    //         link_secret_blinding_data:,
    //         nonce,
    //         link_secret_name: "main_link_secret".to_string(),
    //     };
    //     println!("credential_request_metadata parsed: {:?}\n", credential_request_metadata_obj_parsed);
    //
    //     let metadata  = include_str!("credential_request_metadata.json");
    //     let credential_definition  = include_str!("rev_reg.json");
    //     let credential_processed = W3cProcess::new();
    //
    //     let link_secret = "76478676084355218042342646808554459621308529141390964754643914061612688503934".to_string();
    //     credential_processed.process_credential(credential,
    //                                             metadata,
    //                                             link_secret,
    //                                             credential_definition,
    //                                             Some(rev_reg_def_obj_parsed)
    //     );
    //
    //
    //
    //
    // }

    

}
