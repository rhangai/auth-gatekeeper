use super::state::State;
use crate::config::Config;
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
	let data = "Aa5M1jXk6aau8KF23t8tVMYFD74LISp5SgVoYqzPJp+pZ9vEML/ZD2nOnbYxIlprVCX9cfF/94ipD7lo4I22yjvinHG+s/RSGXTptnSl+k36seqFxxD518GqU+Z7ZDI=";
	let decrypted = state.crypto.decrypt(data)?;
	Ok(HttpResponse::Ok().body(decrypted))
}

async fn validate(state: web::Data<State>) -> Result<impl Responder, Error> {
	Ok(HttpResponse::Ok().body("Hello world!"))
}

pub struct Handler {
	config: Config,
}

impl Handler {
	pub fn new(config: Config) -> Handler {
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
