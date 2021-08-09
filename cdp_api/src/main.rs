//! Implements the services the API responds to.

mod config;
mod db;
mod api;

use crate::api::Api;
use crate::db::inmem::InMemoryApiDatabase;

/// API entry point. Read config, connect to database, and setup services.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  // first, load up config
  let cfg = config::load_defaults()
    .unwrap_or_else(|e| panic!("Configuration tragedy: {:#?}", e));
  // now, load up the database.
  let db = InMemoryApiDatabase::default();
  // init the API struct!
  let api = Api {
    config: cfg,
    db_config: (),
    db: db,
  };
  return api.run_server().await;
}
