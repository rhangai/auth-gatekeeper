use super::data::Data;
use super::state::State;
use crate::config::Config;
use crate::error::Error;
use crate::util::crypto;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginQuery {
	url: Option<String>,
}

///
/// Perform the login
///
async fn login(
	data: web::Data<Data>,
	query: web::Query<LoginQuery>,
) -> Result<impl Responder, Error> {
	let state_str = State::serialize_state(&data.crypto, query.url.clone())?;
	let url = data.provider.get_authorization_url(state_str);
	let response = HttpResponse::TemporaryRedirect()
		.header("location", url)
		.finish();
	Ok(response)

	// let mut builder = HttpResponse::Ok();
	// let encrypted = state.crypto.encrypt("10")?;
	// builder.cookie(
	// 	cookie::Cookie::build(
	// 		state.config.cookie_access_token_name.to_owned(),
	// 		encrypted.to_owned(),
	// 	)
	// 	.path("/")
	// 	.finish(),
	// );
	// builder.cookie(
	// 	cookie::Cookie::build(
	// 		state.config.cookie_refresh_token_name.to_owned(),
	// 		encrypted.to_owned(),
	// 	)
	// 	.path("/")
	// 	.finish(),
	// );
	// Ok(builder.finish())
}

///
/// Callback
///
async fn callback(data: web::Data<Data>, req: HttpRequest) -> Result<impl Responder, Error> {
	data.provider.grant_authorization_code().await?;
	Ok(HttpResponse::Ok().finish())
}

///
/// Validate the login
///
async fn validate(data: web::Data<Data>) -> Result<impl Responder, Error> {
	Ok(HttpResponse::Ok().finish())
}

pub struct Handler {
	random: crypto::RandomPtr,
	config: Config,
}

impl Handler {
	///
	/// Create a new handler
	///
	pub fn new(config: Config) -> Result<Handler, Error> {
		let random = crypto::Crypto::create_random();
		Ok(Handler {
			random: random,
			config: config,
		})
	}

	///
	/// Configure the service
	///
	pub fn config(&self, service_config: &mut web::ServiceConfig) -> Result<(), Error> {
		let data = Data::new(self.config.clone(), self.random.clone())?;
		service_config
			.data(data)
			.route("/login", web::get().to(login))
			.route("/callback", web::get().to(callback))
			.route("/validate", web::get().to(validate));
		Ok(())
	}
}
