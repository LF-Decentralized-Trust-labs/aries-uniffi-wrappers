use error::ErrorCode;
use indy_vdr::{
    common::error::{VdrError, VdrResultExt},
    config::PoolConfig,
    pool::ProtocolVersion,
    utils::Validatable,
};
use once_cell::sync::Lazy;
use std::sync::RwLock;

#[macro_use]
mod macros;

pub mod error;
pub mod ledger;
pub mod pool;
pub mod requests;

static POOL_CONFIG: Lazy<RwLock<PoolConfig>> = Lazy::new(|| RwLock::new(PoolConfig::default()));

#[uniffi::export]
pub fn set_default_logger() -> Result<(), ErrorCode> {
    env_logger::try_init().map_err(|e| ErrorCode::Unexpected {
        error_message: format!("{}", e),
    })?;
    Ok(())
}

#[uniffi::export]
pub fn set_config(config: String) -> Result<(), ErrorCode> {
    let config: PoolConfig =
        serde_json::from_str(config.as_str()).with_input_err("Error deserializing config")?;
    config.validate().map_err(|err| VdrError::from(err))?;
    let mut gcfg = write_lock!(POOL_CONFIG)?;
    *gcfg = config;
    Ok(())
}

#[uniffi::export]
pub fn set_protocol_version(version: i64) -> Result<(), ErrorCode> {
    let version = ProtocolVersion::try_from(version)?;
    let mut gcfg = write_lock!(POOL_CONFIG)?;
    gcfg.protocol_version = version;
    Ok(())
}
