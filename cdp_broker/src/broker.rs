//! Implements functions related to communicating with the API, and abstracts
//! away the whole "Broker" inner state.

use std::sync::Arc;
use std::convert::TryFrom;
use std::thread;

use chrono::{DateTime, Local};
use libcdp::comm::broker_api::{BrokerMessage, BrokerMessageBundle, BrokerMessagePayload, HeartbeatMessage};
use libcdp::comm::sensor_broker::{AnySensorMessage, SensorType};

use reqwest::{Client, Response};
use tokio::sync::mpsc::error::SendError;
use crate::config::BrokerConfig;
use tokio::sync::{Mutex, MutexGuard};
use tokio::sync::mpsc::{self, Receiver, Sender};

/// the entire state of the broker.
#[derive(Debug)]
pub(crate) struct Broker {
  /// Broker config.
  pub(crate) cfg: BrokerConfig,
  /// Configuration for rumqqtd.
  pub(crate) rumqttd_cfg: librumqttd::Config,
  /// Time of last successful exchange of data.
  pub(crate) last_seen: Mutex<Option<DateTime<Local>>>,
  /// Message queue for sending home when ready.
  message_comm: (Sender<BrokerMessage>, Arc<Mutex<Receiver<BrokerMessage>>>),
  /// Message bundle within. Thread-safe.
  message_bundle: Arc<Mutex<BrokerMessageBundle>>
}

impl From<(BrokerConfig, librumqttd::Config)> for Broker {
  fn from((bc, rc): (BrokerConfig, librumqttd::Config)) -> Self {
    let (s, r) = mpsc::channel(bc.bundle_size * bc.buffer_size_bundles);
    return Self {
      cfg: bc,
      rumqttd_cfg: rc,
      last_seen: Mutex::new(None),
      message_comm: (s, Arc::new(Mutex::new(r))),
      message_bundle: Arc::new(Mutex::new(BrokerMessageBundle::new())),
    };
  }
}

impl Broker {
  /// Private function, used by other ones to update the last_seen cell.
  async fn update_last_seen(&self) {
    let mut ls = self.last_seen.lock().await;
    ls.replace(Local::now());
  }

  /// Expose a sender pipe so other threads can give us stuff to send.
  pub(crate) fn get_queue_sender(&self) -> Sender<BrokerMessage> {
    return self.message_comm.0.clone();
  }

  /// Handles an HTTP response from the API. Returns only success responses.
  async fn handle_response(&self, maybe_resp: Result<Response, reqwest::Error>)
  -> Option<Response> {
    if let Ok(resp) = maybe_resp {
      if resp.status().is_success() {
        self.update_last_seen().await;
        return Some(resp);
      } else {
        eprintln!("Got a non-2xx response:\n{:#?}", resp);
      }
    }
    return None;
  }

  /// Send a small request to the API to see if it's up.
  pub(crate) async fn heartbeat(&self) -> bool {
    let tgt = self.cfg.endpoint.join("/heartbeat").expect("Bad endpoint URL?");
    let client = reqwest::Client::new();
    let maybe_resp = client
      .post(tgt)
      .json(&HeartbeatMessage::from(&self.cfg))
      .send()
      .await;
    return self.handle_response(maybe_resp).await.is_some();
  }

  /// Used to acquire a full-on lock on the message bundle.
  async fn lock_bundle(&self) -> MutexGuard<'_, BrokerMessageBundle> {
    return self.message_bundle.lock().await;
  }

  /// Enqueue a message.
  async fn enqueue(&self, payload: BrokerMessagePayload)
  -> Result<(), SendError<BrokerMessage>> {
    let msg = BrokerMessage::construct(self.cfg.uid, payload);
    return self.get_queue_sender().send(msg).await;
  }

  /// Sends a message bundle to API. Called on a timer, or when receiver size
  /// reaches 10. Must be nice. We don't clear the bundle.
  /// It's up to the caller.
  async fn send_bundle(&self, require_size: bool) -> bool {
    let bnd = self.lock_bundle().await;
    if require_size && bnd.len() < self.cfg.bundle_size { return true };
    let tgt = self.cfg.endpoint.join("/bundle").expect("Bad endpoint URL?");
    let cl = Client::new();
    let maybe_resp = cl
      .post(tgt)
      .json(&bnd as &BrokerMessageBundle)
      .send()
      .await;
    return self.handle_response(maybe_resp).await.is_some();
  }

  /// Starts the broker, main timers, and everything.
  pub(crate) async fn start(broker: Arc<Self>) {
    // start up the broker
    let (mut router, console, servers, builder)
      = librumqttd::async_locallink::construct_broker(
        broker.rumqttd_cfg.clone()
      );
    thread::spawn(move || {
      router.start().unwrap();
    });
    // init a tokio runtime to capture messages
    let mut rt = tokio::runtime::Builder::new_multi_thread();
    rt.enable_all();
    rt.build().unwrap().block_on(async {
      let (mut tx, mut rx) = builder
        .connect("localclient", 200)
        .await
        .unwrap();
      // subscribe to known topics.
      for st in SensorType::all_types() {
        tx.subscribe(std::iter::once(st.to_string())).await.unwrap();
      }
      // no idea what this does, honestly
      let console_task = tokio::spawn(console);
      // clone some references to the broker...
      let broker1 = broker.clone();
      let broker2 = broker.clone();
      let broker3 = broker.clone();
      // message decode loop. must be fast. another thread will deal with
      // the data, and sending it home.
      let msg_decode_task = tokio::spawn(async move {
        loop {
          let msg = rx.recv().await;
          if let Err(e) = msg {
            eprintln!("LinkError when recv'ing message: {}", e.to_string());
          } else {
            let data = msg.unwrap();
            let maybe_st = SensorType::try_from(data.topic.as_str());
            if let Ok(st) = maybe_st {
              if broker1.cfg.topics.contains(&st) {
                // yeah we care about this. showtime!
                let mut pbytes: Vec<u8> = Vec::new();
                for b in data.payload {
                  pbytes.extend(b);
                }
                let msg = AnySensorMessage::decode(&data.topic, &pbytes);
                match msg {
                  Ok(pl) => {
                    println!(
                      "Got {} data from sensor #{}!",
                      data.topic,
                      pl.sensor_id()
                    );
                    let sd = BrokerMessagePayload::SensorData(pl);
                    if let Err(se) = broker1.enqueue(sd).await {
                      eprintln!(
                        "Failed to enqueue {} data: {}",
                        data.topic,
                        se
                      );
                    }
                  },
                  Err(dec) => {
                    eprintln!("Sensor sent bad data: {}.", dec);
                  },
                };
              }
            } else {
              // bad sensor topic
              eprintln!("Some sensor sent us a bad topic: \"{}\"", &data.topic);
            }
          }
        }
      });
      // message capture thread. reads messages from comm and puts them into
      // the bundle for sending home.
      let msg_bundle_task = tokio::spawn(async move {
        let mut receiver = (broker2.clone().message_comm.1).clone().lock_owned().await;
        loop {
          let msg = receiver.recv().await.expect("Inner channel closed!");
          let mut bnd = broker2.lock_bundle().await;
          bnd.push(msg);
          if broker2.send_bundle(true).await {
            bnd.clear();
          }
        }
      });
      // message autosend thread. ensures we won't wait forever with a
      // non-full bundle.
      let msg_autosend_task = tokio::spawn(async move {
        loop {
          tokio::time::sleep(
            broker3.cfg.bundle_timeout
              .to_std().expect("Bundle timeouts can't be negative!")
          ).await;
          let mut bnd = broker3.lock_bundle().await;
          if broker3.send_bundle(false).await {
            bnd.clear();
          }
        }
      });
      // wait on all handles. that should be forever unless... yeah.
      println!("Broker is up.");
      if broker.heartbeat().await {
        println!("API seems to be up.");
      } else {
        println!("API seems to be down? Better look into that.");
      }
      servers.await;
      msg_decode_task.await.unwrap();
      msg_bundle_task.await.unwrap();
      msg_autosend_task.await.unwrap();
      console_task.await.unwrap();
    });
  }
}
