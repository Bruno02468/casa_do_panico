//! Messages between brokers and APIs.

use std::error::Error as StdError;
use std::fmt::Display;

use chrono::{DateTime, Local};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::comm::sensor_broker::AnySensorMessage;


/// A heartbeat message. Carries key and uuid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeartbeatMessage {
  /// The unique id of the broker.
  pub uid: Uuid,
  /// The API access secret key.
  pub key: Option<String>
}

/// Payload that can be sent upstream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BrokerMessagePayload {
  /// Message is sensor data.
  SensorData(AnySensorMessage),
  /// Message is a mere heartbeat. Will send key and uuid for checking.
  Heartbeat(HeartbeatMessage)
}

/// Type of payload that can be sent upstream.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BrokerMessagePayloadType {
  SensorData,
  Heartbeat
}

impl Display for BrokerMessagePayloadType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", match self {
      BrokerMessagePayloadType::SensorData => "sensor_data",
      BrokerMessagePayloadType::Heartbeat => "heartbeat"
    })
  }
}

impl From<&BrokerMessagePayload> for BrokerMessagePayloadType {
  fn from(pl: &BrokerMessagePayload) -> Self {
    return match pl {
      BrokerMessagePayload::SensorData(_) => Self::SensorData,
      BrokerMessagePayload::Heartbeat(_) => Self::Heartbeat,
    }
  }
}

/// Message to be sent upstream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrokerMessage {
  /// When this message was constructed. Set by the broker.
  pub constructed_when: DateTime<Local>,
  /// When this message was sent.
  pub sent_when: Option<DateTime<Local>>,
  /// When this message was received. Set by the API.
  pub received_when: Option<DateTime<Local>>,
  /// A copy of the broker unique ID.
  pub broker_id: Uuid,
  /// The payload.
  pub payload: BrokerMessagePayload
}

impl BrokerMessage {
  /// Construct a BrokerMessage from the viewpoint of the broker.
  pub fn construct(broker_id: Uuid, payload: BrokerMessagePayload) -> Self {
    return Self {
      constructed_when: Local::now(),
      sent_when: None,
      received_when: None,
      broker_id: broker_id,
      payload: payload,
    }
  }
  /// Returns the payload type.
  pub fn payload_type(&self) -> BrokerMessagePayloadType {
    return (&self.payload).into();
  }
}

/// A bundle of messages to be sent upstream.
pub type BrokerMessageBundle = Vec<BrokerMessage>;

/// Any error that can occur when phoning home.
#[derive(Debug)]
pub enum UpstreamCommError {
  /// A network/HTTP error, caught by reqwest.
  Net(reqwest::Error),
  /// An API error, detected by a non-2xx status code.
  Api(Response),
  /// Some other error.
  Other(Box<dyn StdError + Send + Sync>)
}
