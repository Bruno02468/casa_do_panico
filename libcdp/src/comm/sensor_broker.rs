//! Messages between sensors and brokers.

use std::convert::TryFrom;
use std::fmt::Display;
use std::error::Error;
use std::str::FromStr;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};

/// Any measurement message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnySensorMessage {
  Temperature(TemperatureMessage),
  Humidity(HumidityMessage)
}

impl AnySensorMessage {
  /// Returns the type of sensor behind the message.
  pub fn sensor_type(&self) -> SensorType {
    return self.into();
  }

  /// Decodes the sensor message from a topic name and a byte sequence.
  pub fn decode<T: AsRef<Vec<u8>>>(topic: &str, data: T)
  -> Result<AnySensorMessage, MessageParseError> {
    return match topic {
      "temperature" => Ok(AnySensorMessage::Temperature(
        TemperatureMessage::try_from(data.as_ref())?
      )),
      "humidity" => Ok(AnySensorMessage::Humidity(
        HumidityMessage::try_from(data.as_ref())?
      )),
      _ => Err(MessageParseError::BadTopic(topic.to_owned()))
    }
  }

  /// Returns the sensor ID within.
  pub fn sensor_id(&self) -> usize {
    return match self {
      AnySensorMessage::Temperature(tm) => tm.get_sensor_id(),
      AnySensorMessage::Humidity(hm) => hm.get_sensor_id(),
    }
  }
}

/// Types of measurement messages.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SensorType {
  Temperature,
  Humidity
}

impl SensorType {
  /// Returns a vector with all types.
  pub fn all_types() -> Vec<Self> {
    return vec![
      Self::Temperature,
      Self::Humidity
    ]
  }
}

impl From<&AnySensorMessage> for SensorType {
  /// Extract the type of sensor from the message.
  fn from(msg: &AnySensorMessage) -> Self {
    return match msg {
      AnySensorMessage::Temperature(_) => Self::Temperature,
      AnySensorMessage::Humidity(_) => Self::Humidity,
    }
  }
}

impl FromStr for SensorType {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for stype in Self::all_types() {
      if stype.to_string() == s {
        return Ok(stype);
      }
    }
    return Err(());
  }
}

impl Display for SensorType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", match self {
      SensorType::Temperature => "temperature",
      SensorType::Humidity => "humidity",
    });
  }
}

/// The kind of error you can get when parsing sensor messages from byte
/// sequences.
#[derive(Debug)]
pub enum MessageParseError {
  /// Bad length: expected first, got last.
  BadLength(usize, usize),
  /// Bad topic name.
  BadTopic(String)
}

impl Error for MessageParseError {}

impl Display for MessageParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      MessageParseError::BadLength(e, g) => {
        write!(f, "Bad length! Expected {}, got {}.", e, g)
      }
      MessageParseError::BadTopic(tn) => {
        write!(f, "Bad topic name \"{}\".", tn)
      },
    };
  }
}

/// The kind of stuff all sensor messages can do.
pub trait SensorMessage: Copy + Clone + std::fmt::Debug
+ TryFrom<Vec<u8>, Error=MessageParseError> + Serialize + DeserializeOwned {
  /// Return the sensor ID as an usize.
  fn get_sensor_id(&self) -> usize;
}

/// Message sent by a temperature sensor.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TemperatureMessage {
  /// Numeric ID of the sensor.
  pub sensor_id: u8,
  /// Temperature value in K.
  pub kelvin: u16
}

impl TryFrom<&Vec<u8>> for TemperatureMessage {
  /// No good though.
  type Error = MessageParseError;
  /// Convert a three-byte sequence into a temperature message.
  fn try_from(data: &Vec<u8>) -> Result<Self, Self::Error> {
    if data.len() != 3 {
      return Err(Self::Error::BadLength(3, data.len()));
    } else {
      let (e1, e2, e3): (u8, u16, u16)
        = (data[0], data[1] as u16, data[2] as u16);
      return Ok(Self {
        sensor_id: e1,
        kelvin: ((e2 as u16) << 8) + e3
      });
    }
  }
}

impl TryFrom<Vec<u8>> for TemperatureMessage {
  type Error = MessageParseError;
  fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
    return Self::try_from(&vec);
  }
}

impl SensorMessage for TemperatureMessage {
  fn get_sensor_id(&self) -> usize {
    return self.sensor_id as usize;
  }
}

/// Message sent by a humidity sensor.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HumidityMessage {
  /// Numeric ID of the sensor.
  pub sensor_id: u8,
  /// Humidity value in relative humidity percentage.
  pub humidity: u8
}

impl TryFrom<&Vec<u8>> for HumidityMessage {
  /// No good though.
  type Error = MessageParseError;
  /// Convert a two-byte sequence into a temperature message.
  fn try_from(data: &Vec<u8>) -> Result<Self, Self::Error> {
    if data.len() != 2 {
      return Err(Self::Error::BadLength(3, data.len()));
    } else {
      let (e1, e2) = (data[0], data[1]);
      return Ok(Self {
        sensor_id: e1,
        humidity: e2
      });
    }
  }
}

impl TryFrom<Vec<u8>> for HumidityMessage {
  type Error = MessageParseError;
  fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
    return Self::try_from(&vec);
  }
}

impl SensorMessage for HumidityMessage {
  fn get_sensor_id(&self) -> usize {
    return self.sensor_id as usize;
  }
}
