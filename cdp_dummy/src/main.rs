//! Entry point for the dummy sensor.

use crate::dummy::{Dummy, DummyConfig};

mod dummy;

fn main() {
  println!("Hey! Loading config...");
  let mut dummy = Dummy::construct(
    DummyConfig::load_defaults()
      .unwrap_or_else(|err| panic!("Configuration tragedy: {}", err)),
    Some(69)
  );
  println!("Configuration loaded! Starting dummy...");
  dummy.start();
  let (oks, fails) = dummy.join();
  println!("Dummy finidhed with {} OK sends and {} failed sends.", oks, fails);
}
