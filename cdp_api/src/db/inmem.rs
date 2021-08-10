//! Implements a simple in-memory database that supports saving and loading
//! through serialization.

use std::collections::HashSet;
use std::error::Error as StdError;
use std::fmt::Display;
use std::iter::FromIterator;

use serde::{Deserialize, Serialize};

use libcdp::comm::broker_api::{BrokerMessage, BrokerMessagePayload, BrokerMessagePayloadType};
use libcdp::comm::sensor_broker::{AnySensorMessage, SensorType};
use std::sync::{Arc, Mutex, PoisonError};

use crate::db::ApiDatabase;

/// The underlying data for the simple in-memory database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct UnderlyingData {
  topics: HashSet<SensorType>,
  messages: Vec<BrokerMessage>
}

impl UnderlyingData {
  /// Creates a new in-memory database, optionally with preloaded topics.
  pub(crate) fn new<T>(iter: T) -> Self where T: Iterator<Item=SensorType> {
    return Self {
      topics: HashSet::from_iter(iter),
      messages: Vec::new()
    }
  }
}

impl Default for UnderlyingData {
  /// The default database has the topics list filled to have all supported
  /// sensor types.
  fn default() -> Self {
    return UnderlyingData::new(SensorType::all_types().into_iter());
  }
}

/// Implements a simple in-memory database that can also be saved and loaded
/// through the magic of serialization.
#[derive(Debug, Clone)]
pub(crate) struct InMemoryApiDatabase {
  backing: Arc<Mutex<UnderlyingData>>
}

impl From<UnderlyingData> for InMemoryApiDatabase {
  fn from(backing: UnderlyingData) -> Self {
    return Self {
      backing: Arc::new(Mutex::new(backing))
    }
  }
}

impl Default for InMemoryApiDatabase {
  fn default() -> Self {
    return Self::from(UnderlyingData::default());
  }
}

/// An error that the memory database can return.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum InMemoryDatabaseError {
  /// A mutex lock died. String is type name.
  PoisonError(String)
}

impl StdError for InMemoryDatabaseError {}

impl Display for InMemoryDatabaseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InMemoryDatabaseError::PoisonError(tn) => {
        return write!(f, "A mutex on a {} was poisoned!", tn);
      }
    }
  }
}

impl<T> From<PoisonError<T>> for InMemoryDatabaseError {
  fn from(_: PoisonError<T>) -> Self {
    return InMemoryDatabaseError::PoisonError(std::any::type_name::<T>().to_owned())
  }
}

impl ApiDatabase for InMemoryApiDatabase {
  type DbError = InMemoryDatabaseError;
  type BrokerMessageIter = Box<dyn Iterator<Item=BrokerMessage>>;
  type SensorMessageIter = Box<dyn Iterator<Item=AnySensorMessage>>;
  type DbConfig = ();

  fn db_type(&self) -> super::ApiDatabaseType {
    return super::ApiDatabaseType::InMemory;
  }

  /// Same as default, I guess.
  fn init(&self, _: Self::DbConfig) -> Result<Self, Self::DbError> {
    return Ok(Self{
      backing: Arc::new(Mutex::new(UnderlyingData::new(None.into_iter())))
    })
  }

  /// No one-time setup needed, I guess
  fn setup(&self) {}

  fn topics(&self) -> Result<HashSet<SensorType>, Self::DbError> {
    let d = self.backing.lock()?;
    return Ok(d.topics.clone());
  }

  fn update_topics<T>(&self, new_topics: T) -> Result<(), Self::DbError>
  where T: IntoIterator<Item=SensorType> {
    let mut d = self.backing.lock()?;
    d.topics.clear();
    d.topics.extend(new_topics);
    return Ok(());
  }

  fn messages_by_type(&self, mtype: BrokerMessagePayloadType)
  -> Result<Self::BrokerMessageIter, Self::DbError> {
    let d = self.backing.lock()?;
    return Ok(Box::new(d.messages
      .clone()
      .into_iter()
      .filter_map(move |m|
        if m.payload_type() == mtype { Some(m.clone()) } else { None }
      )
    ));
  }

  fn sensor_messages_by_type(&self, stype: SensorType)
  -> Result<Self::SensorMessageIter, Self::DbError> {
    let d = self.backing.lock()?;
    return Ok(Box::new(d.messages
      .clone()
      .into_iter()
      .filter_map(move |msg| match msg.payload {
        BrokerMessagePayload::SensorData(sd) => {
          if sd.sensor_type() == stype { Some(sd) } else { None }
        }
        _ => None,
      })
    ));
  }

  fn insert_message(&self, msg: BrokerMessage) -> Result<(), Self::DbError> {
    let mut d = self.backing.lock()?;
    d.messages.push(msg);
    return Ok(());
  }
}
