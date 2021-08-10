//! Abstracts away interaction with the database.

pub(crate) mod inmem;

use std::collections::HashSet;
use std::error::Error as StdError;
use std::fmt::Display;
use std::str::FromStr;

use serde::{Serialize, Deserialize};

use libcdp::comm::broker_api::{BrokerMessage, BrokerMessagePayloadType};
use libcdp::comm::sensor_broker::{AnySensorMessage, SensorType};

/// Trait implemented by all types used to implement database abstractions.
pub(crate) trait ApiDatabase: Sized + Send + Sync + Clone {
  /// The type used when returning broker messages.
  type BrokerMessageIter: Iterator<Item=BrokerMessage>;
  /// The type used when returning sensor messages.
  type SensorMessageIter: Iterator<Item=AnySensorMessage>;
  /// Error returned by the database lib, or by us.
  type DbError: StdError + Send + Sync;
  /// Configuration used by the database.
  type DbConfig: Send + Sync;

  /// Returns the name of the type of database in use.
  fn db_type(&self) -> ApiDatabaseType;
  /// Initialize the connection to the database, if any.
  fn init(&self, cfg: Self::DbConfig) -> Result<Self, Self::DbError>;
  /// Set up the database with the tables and stuff if need be. No harm in
  /// calling it needlessly, but try to be aware -- it might be costly.
  fn setup(&self);
  /// Return topics we care about.
  fn topics(&self) -> Result<HashSet<SensorType>, Self::DbError>;
  /// Update the list of topics we care about.
  fn update_topics<T>(&self, new_topics: T) -> Result<(), Self::DbError>
  where T: IntoIterator<Item=SensorType>;
  /// Get all broker messages of a certain type.
  fn messages_by_type(&self, mtype: BrokerMessagePayloadType)
  -> Result<Self::BrokerMessageIter, Self::DbError>;
  /// Get all sensor messages of a certain sensor type.
  fn sensor_messages_by_type(&self, stype: SensorType)
  -> Result<Self::SensorMessageIter, Self::DbError>;
  /// Insert a message into the database.
  fn insert_message(&self, msg: BrokerMessage) -> Result<(), Self::DbError>;
}

/// Types of available API databases.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ApiDatabaseType {
  InMemory
}

impl ApiDatabaseType {
  pub(crate) fn all_types() -> Vec<Self> {
    return vec![
      ApiDatabaseType::InMemory
    ];
  }
}

impl Display for ApiDatabaseType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match self {
      ApiDatabaseType::InMemory => "in_memory",
    })
  }
}

impl FromStr for ApiDatabaseType {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for dbtype in Self::all_types() {
      if dbtype.to_string() == s {
        return Ok(dbtype);
      }
    }
    return Err(());
  }
}
