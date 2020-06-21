use super::state::State;
use actix_web::{web, HttpResponse, Responder};

async fn login() -> impl Responder {
	HttpResponse::Ok().body("Hello world!")
}

async fn callback() -> impl Responder {
	HttpResponse::Ok().body("Hello world again!")
}

async fn validate() -> impl Responder {
	HttpResponse::Ok().body("Hello world again!")
}

pub struct Handler {}

impl Handler {
	pub fn new() -> Handler {
		Handler {}
	}

	// this function could be located in different module
	pub fn config(&self, cfg: &mut web::ServiceConfig) {
		cfg.route("/login", web::get().to(login))
			.route("/callback", web::get().to(callback))
			.route("/validate", web::get().to(validate));
	}
}
