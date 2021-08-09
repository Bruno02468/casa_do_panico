//! Implement request handlers for the API.

use actix_web::{web, HttpResponse};

use crate::db::ApiDatabase;

pub(crate) async fn index<D: ApiDatabase>(_: web::Data<D>) -> HttpResponse {
  return HttpResponse::Ok().body("API up!");
}
