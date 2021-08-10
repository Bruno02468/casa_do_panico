//! Entry point for the dummy sensor.

use std::thread::{self, JoinHandle};

use crate::dummy::Dummy;

mod config;
mod dummy;

fn main() {
  println!("Hey! Loading config...");
  let configs = config::load_multi()
    .unwrap_or_else(|err| panic!("Configuration tragedy: {}", err));
  let mut dummies: Vec<JoinHandle<Dummy>> = Vec::new();
  println!("Configuration loaded! Starting {} dummies...", dummies.len());
  for (i, cfg) in configs.iter().enumerate() {
    let mut dummy = Dummy::construct(cfg.clone(), Some(i as u8));
    let jh = thread::spawn(move || { (&mut dummy).start(); dummy });
    dummies.push(jh);
  }
  let (mut oks, mut fails): (usize, usize) = (0, 0);
  for dummy in dummies {
    let (doks, dfails) = dummy.join().unwrap().join();
    oks += doks;
    fails += dfails;
  }
  println!("All dummies finidhed! Sent {} and failed {}.", oks, fails);
}
