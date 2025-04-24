pub mod crypto;
pub mod entry;
pub mod error;
pub mod key;
pub mod scan;
pub mod session;
pub mod store;
pub mod tags;

#[uniffi::export]
pub fn set_default_logger() -> Result<(), error::ErrorCode> {
    env_logger::try_init().map_err(|e| error::ErrorCode::Unexpected {
        error_message: format!("{}", e),
    })?;
    Ok(())
}
