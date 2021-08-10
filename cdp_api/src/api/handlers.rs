//! Implement request handlers for the API.

use actix_web::{web, HttpResponse};
use chrono::Local;
use libcdp::comm::broker_api::{BrokerMessage, BrokerMessageBundle, BrokerMessagePayloadType};

use crate::db::ApiDatabase;

/// Handles request to /. Nothing special.
pub(crate) async fn index<D: ApiDatabase>(_: web::Data<D>)
-> HttpResponse {
  return HttpResponse::Ok().body("API up!");
}

/// We'll do a lil' checkin' later.
pub(crate) async fn heartbeat<D: ApiDatabase>(_: web::Data<D>)
-> HttpResponse {
  return HttpResponse::Ok().body("OK");
}

/// Pushes the message bundle to the database.
pub(crate) async fn bundle<D: ApiDatabase>(
  msgs: web::Json<BrokerMessageBundle>, db: web::Data<D>
) -> HttpResponse {
  for mut msg in msgs.into_inner() {
    msg.received_when = Some(Local::now());
    match db.insert_message(msg) {
      Ok(_) => continue,
      Err(_) => return HttpResponse::InternalServerError().body("god damnit"),
    };
  } 
  return HttpResponse::Ok().body("OK")
}

/// Returns all messages.
pub(crate) async fn all_sensor<D: ApiDatabase>(db: web::Data<D>)
-> HttpResponse {
  let msgs: Vec<BrokerMessage> = db
    .messages_by_type(BrokerMessagePayloadType::SensorData)
    .unwrap()
    .collect();
  return HttpResponse::Ok().json(msgs);
}

