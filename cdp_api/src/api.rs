//! Abstracts away inner API state and config.

mod handlers;

use actix_web::{App, HttpServer, web};

use crate::config::ApiConfig;
use crate::db::ApiDatabase;

/// Contains the whole state of the API.
#[derive(Clone)]
pub(crate) struct Api<D: ApiDatabase> {
  /// Configuration loaded from files.
  pub(crate) config: ApiConfig,
  /// Database configuration.
  pub(crate) db_config: D::DbConfig,
  /// API database connection.
  pub(crate) db: D
}

impl<D: ApiDatabase + 'static> Api<D> {
  /// Say something generic when people hit up /.
  pub(crate) async fn run_server(&self) -> std::io::Result<()> {
    // init server
    let dbc = self.db.clone();
    let mut srv = HttpServer::new(move || {
      App::new()
        .data(dbc.clone())
        .route("/", web::get().to(handlers::index::<D>))
    });
    // bind to cfg'd addrs
    for addr in self.config.binds.iter() {
      println!("Binding to {}...", &addr);
      srv = srv.bind(addr)?;
    }
    // showtime!
    println!("API is up!");
    return srv.run().await;
  }
}
