use super::state::State;
use crate::config;
use crate::error::Error;
use crate::util::crypto;
use actix_web::{cookie, web, HttpMessage, HttpRequest, HttpResponse, Responder};

async fn login(state: web::Data<State>) -> Result<impl Responder, Error> {
	let mut builder = HttpResponse::Ok();
	let encrypted = state.crypto.encrypt("10")?;
	builder.cookie(
		cookie::Cookie::build(
			state.config.cookie_access_token_name.to_owned(),
			encrypted.to_owned(),
		)
		.path("/")
		.finish(),
	);
	builder.cookie(
		cookie::Cookie::build(
			state.config.cookie_refresh_token_name.to_owned(),
			encrypted.to_owned(),
		)
		.path("/")
		.finish(),
	);
	Ok(builder.finish())
}

async fn callback(req: HttpRequest, state: web::Data<State>) -> Result<impl Responder, Error> {
	let data = "Aac5HK7PFg9bdyPm5TTItam2Pz/Dqfxqi7pt6u9qDKkuxNBRQV2QZyLVcjj4rWxeDvtdqzdnUuMjnBIfRvtzrgqsOwnW69zn19RMUUkfKCi1KjCYuElOUpSbtyu7CBE=";
	let decrypted = state.crypto.decrypt(data)?;
	Ok(HttpResponse::Ok().body("Hello world!"))
}

async fn validate(state: web::Data<State>) -> Result<impl Responder, Error> {
	Ok(HttpResponse::Ok().body("Hello world!"))
}

pub struct Handler {
	config: config::Config,
}

impl Handler {
	pub fn new() -> Handler {
		let config = config::Config::new();
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
