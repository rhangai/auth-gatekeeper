use super::state::State;
use crate::config;
use crate::error::Error;
use crate::util::crypto;
use actix_web::{web, HttpResponse, Responder};

async fn login(state: web::Data<State>) -> Result<impl Responder, Error> {
	Ok(HttpResponse::Ok().body("Hello world!"))
}

async fn callback(state: web::Data<State>) -> impl Responder {
	HttpResponse::Ok().body("Hello world again!")
}

async fn validate(state: web::Data<State>) -> impl Responder {
	HttpResponse::Ok().body("Hello world again!")
}

pub struct Handler {
	config: config::Config,
}

impl Handler {
	pub fn new() -> Handler {
		let config = config::Config {
			cookie_secret: String::from("oi"),
		};
		Handler { config: config }
	}

	pub fn config(&self, service_config: &mut web::ServiceConfig) {
		let random = crypto::Crypto::create_random();
		let state = State::new(self.config.clone(), random);
		service_config
			.data(state)
			.route("/login", web::get().to(login))
			.route("/callback", web::get().to(callback))
			.route("/validate", web::get().to(validate));
	}
}
