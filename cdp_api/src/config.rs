//! Implements configuration for the API.

use std::convert::{TryFrom, TryInto};
use std::error::Error;

use config::{Config, ConfigError};
use serde::{Serialize, Deserialize};

/// Encodes the information in an API config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiConfigFile {
  /// List of address:port items to bind to.
  /// Note that IPv4 and IPv6 are to be specified separately.
  binds: Vec<String>
}

impl Default for ApiConfigFile {
  /// Default binds all:9869.
  fn default() -> Self {
    return Self {
      binds: vec![
        "0.0.0.0:9869".to_owned(),
        "[::]:9869".to_owned()
      ]
    }
  }
}

/// The decoded, properly-parsed version of the ApiConfigFile struct.
#[derive(Debug, Clone)]
pub(crate) struct ApiConfig {
  /// List of address:port items to bind to.
  /// Note that IPv4 and IPv6 are to be specified separately.
  pub(crate) binds: Vec<String>
}

#[derive(Debug)]
pub(crate) enum ApiConfigParseError {
  /// Parse error from the config crate.
  ConfigError(ConfigError),
  /// Parse error from our conversion. Dead code allowed because this doesn't
  /// have the abilit to fail yet, but it might, in the future.
  #[allow(dead_code)]
  ParseError(Box<dyn Error + Send + Sync>)
}

impl From<ConfigError> for ApiConfigParseError {
  fn from(cfgerr: ConfigError) -> Self {
    return Self::ConfigError(cfgerr)
  }
}

impl TryFrom<ApiConfigFile> for ApiConfig {
  /// Generic error type for when the conversion fails.
  type Error = ApiConfigParseError;

  /// Fallible parsing. No errors now... but who knows?
  fn try_from(pre: ApiConfigFile) -> Result<Self, Self::Error> {
    return Ok(Self {
      binds: pre.binds
    });
  }
}

impl Default for ApiConfig {
  /// Default binds all:9869.
  fn default() -> Self {
    return Self::try_from(ApiConfigFile::default())
      .expect("Default config failed to parse!");
  }
}

/// Load the default configuration files for the API.
pub(crate) fn load_defaults() -> Result<ApiConfig, ApiConfigParseError> {
  let mut cfg = Config::default();
  cfg.merge(config::File::with_name("cdp_api"))?;
  let api_cfg: ApiConfigFile = cfg.try_into()?;
  return Ok(api_cfg.try_into()?);
}
