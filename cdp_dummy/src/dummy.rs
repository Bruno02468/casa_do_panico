//! Implements a single dummy sensor.

use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::str::FromStr;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use config::{Config, ConfigError};
use libcdp::comm::sensor_broker::SensorType;
use rand::Rng;
use serde::{Serialize, Deserialize};
use rumqttc::{MqttOptions, Client, QoS};

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
  fn gen_interval(&self) -> Duration {
    let range = 0u128 .. self.interval_jitter.as_millis(); 
    let mut jitter = rand::thread_rng().gen_range(range) as i128;
    if rand::thread_rng().gen_bool(0.5) { jitter *= -1; };
    let total = jitter + self.interval.as_millis() as i128;
    let clamped: u128 = if total < 0 { 0 } else { total as u128 };
    return Duration::from_millis(
      clamped.try_into().expect("Bad jitter produced too big a duration!")
    );
  }

  /// Generate a random payload. Optionally override first byte (ID).
  fn gen_payload(&self, id_override: Option<u8>) -> Vec<u8> {
    let index = rand::thread_rng().gen_range(0..self.payloads.len());
    let mut payload = self.payloads.get(index).unwrap().clone();
    if let Some(b) = id_override {
      if payload.len() > 0 {
        payload.remove(0);
        payload.insert(0, b);
      }
    }
    return payload;
  }

  /// Load the default dummy configuration from files.
  pub(crate) fn load_defaults() -> Result<DummyConfig, DummyConfigError> {
    let mut cfg = Config::default();
    cfg.merge(config::File::with_name("cdp_dummy"))?;
    let dummy_cfg: DummyConfigFile = cfg.try_into()?;
    return Ok(dummy_cfg.try_into()?);
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
          let mut vec = v.to_ne_bytes().to_vec();
          vec.truncate(bl.into());
          while vec.len() < bl.into() {
            vec.insert(0, 0);
          }
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

/// A dummy and its whole state.
pub(crate) struct Dummy {
  /// A copy of the dummy config.
  pub(crate) cfg: DummyConfig,
  /// A byte to override the first byte of payloads (sensor ID).
  pub(crate) id_override: Option<u8>,
  /// A handle for the inner thread. Counts ok and fails.
  thread: Option<JoinHandle<(usize, usize)>>
}

impl Dummy {
  /// Construct a dummy.
  pub(crate) fn construct(cfg: DummyConfig, id_override: Option<u8>) -> Self {
    return Self {
      cfg: cfg,
      id_override: id_override,
      thread: None
    }
  }

  /// Returns true if the join handle is started.
  pub(crate) fn is_running(&self) -> bool {
    return self.thread.is_some();
  }
  
  /// Starts this dummy's thread and sets up the join handle.
  pub(crate) fn start(&mut self) {
    if self.is_running() { return; }
    let cfg = self.cfg.clone();
    let cid = self.id_override.clone();
    let mut opts = MqttOptions::new(
      format!("dummy-{}", cid.unwrap_or(0)),
      &cfg.broker_address,
      cfg.broker_port
    );
    opts.set_keep_alive(5);
    let (mut client, mut cxn) = Client::new(opts, 10);
    self.thread = Some(thread::spawn(move || {
      let (mut oks, mut fails): (usize, usize) = (0, 0);  
      loop {
        let pld = cfg.gen_payload(cid);
        println!("payload: {:?}", pld);
        let res = client.publish(
          cfg.topic.to_string(),
          QoS::AtMostOnce,
          false,
          pld
        );
        match res {
          Ok(_) => {
            println!(
              "Sent {} data to the broker successfully!",
              &cfg.topic
            );
            oks += 1;
          },
          Err(ce) => {
            eprintln!(
              "Failed to send data (ClientError): {}", &ce
            );
            eprintln!("{:#?}", ce);
            fails += 1;
            if fails > 10 { break; }
          },
        };
        thread::sleep(cfg.gen_interval());
      }
      return (oks, fails);
    }));
    println!("Dummy started!");
    let mut cxn_errs = 0;
    for (i, nxn) in cxn.iter().enumerate() {
      println!("Notification #{}: {:?}", i+1, nxn);
      if nxn.is_err() {
        cxn_errs += 1;
        if cxn_errs > 10 {
          break;
        }
      }
    }
    panic!("Dummy stopped due to errs!");
  }

  /// Wait on the dummy.
  pub(crate) fn join(&mut self) -> (usize, usize) {
    if self.thread.is_some() {
      let jh = self.thread.take().unwrap();
      return jh
        .join()
        .expect("Could not acquire JoinHandle result! Did the thread die?");
    } else {
      return (0, 0);
    }
  }
}
