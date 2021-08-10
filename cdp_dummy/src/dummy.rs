//! Implements a single dummy sensor.

use std::thread::{self, JoinHandle};
use rumqttc::{MqttOptions, Client, QoS};

use crate::config::DummyConfig;

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
    let idname = cid.map(|s| s.to_string()).unwrap_or("?".to_owned());
    let name = format!("dummy-{}", idname);
    let outername = name.clone();
    let mut opts = MqttOptions::new(
      name.to_owned(),
      &cfg.broker_address,
      cfg.broker_port
    );
    opts.set_keep_alive(5);
    let (mut client, mut cxn) = Client::new(opts, 10);
    self.thread = Some(thread::spawn(move || {
      let (mut oks, mut fails): (usize, usize) = (0, 0);  
      let mut rng = rand::thread_rng();
      loop {
        let pld = cfg.gen_payload(cid, &mut rng);
        let res = client.publish(
          cfg.topic.to_string(),
          QoS::AtMostOnce,
          false,
          pld
        );
        match res {
          Ok(_) => {
            println!(
              "[{}] Sent {} data to the broker successfully!",
              &name,
              &cfg.topic
            );
            oks += 1;
          },
          Err(ce) => {
            eprintln!(
              "[{}] Failed to send data (ClientError): {}",
              name,
              &ce
            );
            fails += 1;
            if fails > 10 { break; }
          },
        };
        thread::sleep(cfg.gen_interval(&mut rng));
      }
      return (oks, fails);
    }));
    println!("[{}] Started!", &outername);
    let mut cxn_errs = 0;
    for (_, nxn) in cxn.iter().enumerate() {
      if nxn.is_err() {
        cxn_errs += 1;
        if cxn_errs > 10 {
          break;
        }
      }
    }
    panic!("[{}] stopped due to max errors!", &outername);
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
