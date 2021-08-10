//! Dummy configuration.

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

use config::{Config, ConfigError};
use libcdp::comm::sensor_broker::SensorType;
use rand::Rng;
use rand::prelude::{SliceRandom, ThreadRng};
use serde::{Serialize, Deserialize};

/// Dummy sensor mode of operation.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum DummyMode {
  /// Constantly output the minimum value in the set range.
  ConstantMin,
  /// Constantly output the maximum value in the set range.
  ConstantMax,
  /// Output random values from the set range.
  Random
}

impl Display for DummyMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", match self {
      DummyMode::ConstantMin => "constant_min",
      DummyMode::ConstantMax => "constant_max",
      DummyMode::Random => "random",
    });
  }
}

impl FromStr for DummyMode {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for mode in Self::all_modes() {
      if mode.to_string() == s {
        return Ok(mode);
      }
    }
    return Err(());
  }
}

impl DummyMode {
  /// Returns all dummy modes.
  pub(crate) fn all_modes() -> Vec<Self> {
    return vec![
      DummyMode::ConstantMin,
      DummyMode::ConstantMax,
      DummyMode::Random
    ];
  }
}

/// Configuration for a single dummy sensor. Read from config file too.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DummyConfigFile {
  /// The target broker's address.
  pub(crate) broker_address: String,
  /// The target broker's port.
  pub(crate) broker_port: u16,
  /// Value selection mode.
  pub(crate) mode: String,
  /// List of values that the dummy can output as (number, bytelen).
  pub(crate) values: Vec<(usize, u8)>,
  /// The topic/sensor type to output.
  pub(crate) topic: String,
  /// The time interval between sends.
  pub(crate) interval_msecs: usize,
  /// A jitter for the interval.
  pub(crate) interval_jitter_msecs: usize
}

impl Default for DummyConfigFile {
  fn default() -> Self {
    return Self {
      broker_address: "localhost".to_owned(),
      broker_port: 9869,
      mode: "random".to_owned(),
      values: Vec::new(),
      topic: "<INSERT TOPIC HERE>".to_owned(),
      interval_msecs: 1000,
      interval_jitter_msecs: 500
    }
  }
}

/// Configuration for a single dummy sensor. Read from config file too.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DummyConfig {
  /// The target broker's address.
  pub(crate) broker_address: String,
  /// The target broker's port.
  pub(crate) broker_port: u16,
  /// The value selection mode, parsed.
  pub(crate) mode: DummyMode,
  /// List of payloads to send.
  pub(crate) payloads: Vec<Vec<u8>>,
  /// The topic/sensor type to output.
  pub(crate) topic: SensorType,
  /// The time interval between sends.
  pub(crate) interval: Duration,
  /// A jitter for the interval.
  pub(crate) interval_jitter: Duration 
}

impl DummyConfig {
  /// Generate an interval based on jitter and such.
  pub(crate) fn gen_interval(&self, rng: &mut ThreadRng) -> Duration {
    let range = 0u128 .. self.interval_jitter.as_millis(); 
    let mut jitter = rng.gen_range(range) as i128;
    if rng.gen_bool(0.5) { jitter *= -1; };
    let total = jitter + self.interval.as_millis() as i128;
    let clamped: u128 = if total < 0 { 0 } else { total as u128 };
    return Duration::from_millis(
      clamped.try_into().expect("Bad jitter produced too big a duration!")
    );
  }

  /// Generate a random payload. Optionally override first byte (ID).
  pub(crate) fn gen_payload(
    &self, id_override: Option<u8>, rng: &mut ThreadRng
  ) -> Vec<u8> {
    let mut payload = self.payloads.choose(rng).unwrap().clone();
    println!("{}, {:#?}", self.payloads.len(), payload);
    if let Some(b) = id_override {
      if payload.len() > 0 {
        payload.remove(0);
        payload.insert(0, b);
      }
    }
    return payload;
  }
}

/// Errors that can be found when parsing a config file.
#[derive(Debug)]
pub(crate) enum DummyConfigError {
  /// Upper error caused by the Config crate.
  ConfigError(ConfigError),
  /// Sensor type string not recognized.
  BadSensorType(String),
  /// Bad value mode.
  BadModeName(String)
}

impl std::error::Error for DummyConfigError {}

impl Display for DummyConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DummyConfigError::ConfigError(ce) => {
        return write!(f, "ConfigError: \"{}\"!", ce);
      },
      DummyConfigError::BadSensorType(s) => {
        return write!(f, "Bad sensor type \"{}\"!", s);
      },
      DummyConfigError::BadModeName(s) => {
        return write!(f, "Bad sensor mode \"{}\"!", s);
      },
    }
  }
}

impl From<ConfigError> for DummyConfigError {
  fn from(ce: ConfigError) -> Self {
    return DummyConfigError::ConfigError(ce);
  }
}

impl TryFrom<DummyConfigFile> for DummyConfig {
  type Error = DummyConfigError;
  fn try_from(cfgf: DummyConfigFile) -> Result<Self, Self::Error> {
    return Ok(Self {
      broker_address: cfgf.broker_address.clone(),
      broker_port: cfgf.broker_port,
      mode: DummyMode::from_str(&cfgf.mode)
        .map_err(|_| DummyConfigError::BadModeName(cfgf.mode.clone()))?,
      payloads: cfgf.values.clone()
        .into_iter()
        .map(|(v, bl): (usize, u8)| {
          // weirdo routine to convert usize to zero-padded Vec<u8>
          let mut vec = v.to_le_bytes().to_vec();
          vec.truncate(bl.into());
          while vec.len() < bl.into() {
            vec.insert(0, 0);
          }
          vec.reverse();
          return vec;
        })
        .collect(),
      topic: SensorType::from_str(&cfgf.topic)
        .map_err(|_| DummyConfigError::BadSensorType(cfgf.topic.clone()))?,
      interval: Duration::from_millis(cfgf.interval_msecs as u64),
      interval_jitter: Duration::from_millis(
        cfgf.interval_jitter_msecs as u64
      ),
    });
  }
}

/// Config file for multiple dummies.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct MultiDummyConfigFile {
  dummies: HashMap<String, DummyConfigFile>
}

impl TryFrom<MultiDummyConfigFile> for Vec<DummyConfig> {
  type Error = DummyConfigError;

  fn try_from(m: MultiDummyConfigFile) -> Result<Self, Self::Error> {
    let mut vec = Self::new();
    for (_, dcf) in m.dummies {
      let dc = DummyConfig::try_from(dcf)?;
      vec.push(dc);
    }
    return Ok(vec);
  }
}

pub(crate) fn load_multi() -> Result<Vec<DummyConfig>, DummyConfigError> {
  let mut cfg = Config::default();
  cfg.merge(config::File::with_name("cdp_dummy"))?;
  let multi: MultiDummyConfigFile = cfg.try_into()?;
  return Ok(multi.try_into()?);
}
