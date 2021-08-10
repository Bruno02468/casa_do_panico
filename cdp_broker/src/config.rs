//! Broker configuration. Loading, structures, etc.

use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::time::Duration;

use libcdp::comm::broker_api::HeartbeatMessage;
use libcdp::comm::sensor_broker::SensorType;
use reqwest::Url;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use config::{Config, ConfigError};
use librumqttd::Config as RumqqtdConfig;

/// The broker config as it lies within the file.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct BrokerConfigFile {
  /// What topics to subscribe to and send home.
  topics: Vec<String>,
  /// Access key for the HTTP target. None means no authentication.
  home_key: Option<String>,
  /// The server to contact when phoning home.
  endpoint: String,
  /// Bundle size for the endpoint. Accumulate messages and send no more than
  /// said amount.
  bundle_size: usize,
  /// Bundle timeout for the endpoint. Ensures messages are sent even if
  /// bundle_size has not been reached.
  bundle_timeout_msec: usize,
  /// Buffer size for the endpoint channel.
  buffer_size_bundles: usize,
  /// Heartbeat interval for the endpoint. None means no auto heartbeat.
  heartbeat_interval_secs: Option<usize>,
  /// This broker's unique identifier. Should be random and static.
  uid: String,
}

/// Now, the broker config after some parsing and checks.
#[derive(Clone, Debug)]
pub struct BrokerConfig {
  /// What topics to subscribe to and send home.
  pub topics: Vec<SensorType>,
  /// Access key for the HTTP target. None means no authentication.
  pub home_key: Option<String>,
  /// The server to contact when phoning home.
  pub endpoint: Url,
  /// Bundle size for the endpoint. Accumulate messages and send no more than
  /// said amount.
  pub bundle_size: usize,
  /// Bundle timeout for the endpoint. Ensures messages are sent even if
  /// bundle_size has not been reached.
  pub bundle_timeout: Duration,
  /// Buffer size for the endpoint channel.
  pub buffer_size_bundles: usize,
  /// Heartbeat interval for the endpoint. None means no auto heartbeat.
  pub heartbeat_interval: Option<Duration>,
  /// This broker's unique identifier. Should be random and static.
  pub uid: Uuid,
}

/// An error that can arise while parsing BrokerConfigFile into BrokerConfig.
#[derive(Debug)]
pub enum BrokerConfigParseError {
  /// Meaning the endpoint URL has a syntax error of some sort.
  BadEndpointUrl(url::ParseError),
  /// Meaning the uuid for the broker was malformed.
  BadBrokerUuid(uuid::Error),
  /// Listed topic is not a valid sensor type.
  BadSensorType(String),
  /// An error caught by the config crate.
  ConfigError(ConfigError)
}

impl From<ConfigError> for BrokerConfigParseError {
  fn from(cfgerr: ConfigError) -> Self {
    return Self::ConfigError(cfgerr)
  }
}

impl Default for BrokerConfigFile {
  /// Returns an example configuration with sane values, good for generating
  /// a brand-new configuration file.
  fn default() -> Self {
    return Self {
      topics: vec![],
      home_key: Some("<ACCESS KEY GOES HERE>".to_owned()),
      endpoint: "<ENDPOINT URL GOES HERE>".to_owned(),
      bundle_size: 10,
      bundle_timeout_msec: 5000,
      buffer_size_bundles: 10,
      heartbeat_interval_secs: Some(30),
      uid: Uuid::new_v4().to_string(),
    }
  }
}

impl BrokerConfigFile {
  /// Returns the endpoint URl, properly parsed (if correct).
  pub fn endpoint_url(&self) -> Result<Url, url::ParseError> {
    return Url::parse(&self.endpoint)
  }
}

impl TryFrom<&BrokerConfigFile> for BrokerConfig {
  type Error = BrokerConfigParseError;
  /// Attempt converting the file-parsed struct into the actual options.
  fn try_from(cfg: &BrokerConfigFile) -> Result<Self, Self::Error> {
    let mut topics: Vec<SensorType> = Vec::new();
    for name in &cfg.topics {
      match SensorType::from_str(name.as_str()) {
        Ok(st) => topics.push(st),
        Err(_) => return Err(
          BrokerConfigParseError::BadSensorType(name.to_owned())
        )
      };
    }
    return Ok(Self {
      topics: topics,
      home_key: cfg.home_key.clone(),
      endpoint: cfg.endpoint_url()
        .map_err(|e| Self::Error::BadEndpointUrl(e))?,
      bundle_size: cfg.bundle_size,
      bundle_timeout: Duration::from_millis(cfg.bundle_timeout_msec as u64),
      buffer_size_bundles: cfg.buffer_size_bundles,
      heartbeat_interval: cfg.heartbeat_interval_secs
        .map(|secs| Duration::from_secs(secs as u64)),
      uid: Uuid::parse_str(&cfg.uid)
        .map_err(|e| Self::Error::BadBrokerUuid(e))?,
    });
  }
}

impl TryFrom<BrokerConfigFile> for BrokerConfig {
  type Error = BrokerConfigParseError;
  /// Attempt converting the file-parsed struct into the actual options.
  fn try_from(cfg: BrokerConfigFile) -> Result<Self, Self::Error> {
    return (&cfg).try_into();
  }
}

impl From<&BrokerConfig> for HeartbeatMessage {
  /// Allow creation of a HeartbeatMessage directly from broker config.
  fn from(cfg: &BrokerConfig) -> Self {
    return Self {
      uid: cfg.uid,
      key: cfg.home_key.clone()
    }
  }
}

/// Load the default configuration files for the broker.
pub fn load_defaults()
-> Result<(BrokerConfig, RumqqtdConfig), BrokerConfigParseError> {
  let mut cfg = Config::default();
  cfg
    .merge(config::File::with_name("cdp_rumqttd"))?
    .merge(config::File::with_name("cdp_broker"))?;
  let bc: BrokerConfigFile = cfg.clone().try_into()?;
  let rc: RumqqtdConfig = cfg.try_into()?;
  return Ok((bc.try_into()?, rc));
}
