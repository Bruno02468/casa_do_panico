//! Main broker module. Entry point and such.

use std::sync::Arc;

use crate::broker::Broker;

mod broker;
mod config;

fn main() {
  println!("Hi! Loading configuration...");
  let (broker_config, rumqttd_config) = config::load_defaults()
    .unwrap_or_else(|e| panic!("Configuration tragedy: {:#?}", e));
  println!("Configuration loaded! Phew. Initializing broker...");
  let broker = Broker::from((broker_config, rumqttd_config));
  futures::executor::block_on(Broker::start(Arc::new(broker)));
}
