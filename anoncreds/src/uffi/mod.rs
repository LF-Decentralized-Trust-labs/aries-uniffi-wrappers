pub mod error;
pub mod issuer;
pub mod prover;
pub mod types;
pub mod verifier;
pub mod credential_conversions;

#[uniffi::export]
pub fn set_default_logger() -> Result<(), error::ErrorCode> {
    env_logger::try_init().map_err(|e| error::ErrorCode::Unexpected {
        error_message: format!("{}", e),
    })?;
    Ok(())
}

#[uniffi::export]
pub fn create_link_secret() -> Result<String, error::ErrorCode> {
    let link_secret = anoncreds::prover::create_link_secret()?;
    let dec_secret = link_secret
        .try_into()
        .map_err(|err| anoncreds::Error::from(err))?;
    Ok(dec_secret)
}