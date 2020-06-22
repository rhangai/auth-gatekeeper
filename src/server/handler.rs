use super::data::Data;
use super::response::Response;
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

#[derive(Deserialize)]
struct CallbackQuery {
	state: Option<String>,
	code: Option<String>,
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
	let response = HttpResponse::Found().header("location", url).finish();
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
async fn callback(
	data: web::Data<Data>,
	query: web::Query<CallbackQuery>,
) -> Result<impl Responder, Error> {
	// No query code, so unauthorized
	if query.code.is_none() {
		return Ok(HttpResponse::Unauthorized().finish());
	}

	// Try to request an access token
	let token_set = data
		.provider
		.grant_authorization_code(&query.code.as_ref().unwrap())
		.await?;
	if token_set.is_none() {
		return Ok(HttpResponse::Unauthorized().finish());
	}

	let mut builder = HttpResponse::Found();

	Response::token_set_add_cookies(&mut builder, &data, &token_set.as_ref().unwrap());

	// Set the location
	{
		let mut location: String = String::from("/");
		if query.state.is_some() {
			let state = State::deserialize_state(&data.crypto, &query.state.as_ref().unwrap());
			if state.is_ok() {
				location = state.unwrap().url.unwrap_or(location);
			}
		}
		builder.header("location", location);
	}
	Ok(builder.finish())
}

///
/// Validate the login
///
async fn validate(data: web::Data<Data>) -> Result<impl Responder, Error> {
	let userinfo = data
		.provider
		.userinfo("eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJfeEoxUGF2T1B1ODF1Y0JpQURBTWtCaDN1VkJfemxXSnVzSi03aE1WU1FrIn0.eyJleHAiOjE1OTI4Mzg3MDksImlhdCI6MTU5MjgzODQwOSwianRpIjoiNTNmNDA2MjMtMWI5Ni00YTIxLWE3ZDktOWM4NzI3NTNiZDkwIiwiaXNzIjoiaHR0cDovL2F1dGguaG9uZXN0LmxvY2FsaG9zdC9hdXRoL3JlYWxtcy9ob25lc3QiLCJhdWQiOiJhY2NvdW50Iiwic3ViIjoiNGU5ZmFiNGItNGMyZi00NTI5LTkwODgtMWE4Njg4ODM0NGZjIiwidHlwIjoiQmVhcmVyIiwiYXpwIjoidGVzdCIsInNlc3Npb25fc3RhdGUiOiIwYTc0ZjczYy1mNjY3LTQ5NWEtYWE5NC0zZjBmY2U2ODY0NmQiLCJhY3IiOiIxIiwiYWxsb3dlZC1vcmlnaW5zIjpbImh0dHA6Ly9sb2NhbGhvc3Q6ODA4OCJdLCJyZWFsbV9hY2Nlc3MiOnsicm9sZXMiOlsib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiJdfSwicmVzb3VyY2VfYWNjZXNzIjp7InRlc3QiOnsicm9sZXMiOlsidW1hX3Byb3RlY3Rpb24iXX0sImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoiZW1haWwgcHJvZmlsZSIsImNsaWVudElkIjoidGVzdCIsImVtYWlsX3ZlcmlmaWVkIjpmYWxzZSwiY2xpZW50SG9zdCI6Ijo6MSIsInByZWZlcnJlZF91c2VybmFtZSI6InNlcnZpY2UtYWNjb3VudC10ZXN0IiwiY2xpZW50QWRkcmVzcyI6Ijo6MSJ9.WxivdqHSfyGxj97xpwN9OMjhTJ6mIIUuApqxk9-5qmo3SUMYpeB5q17atxG1ZQtxjUMf_NkHNSd33SqS7PPJUsajND6ix2zvzhzFRjGBiNBuifPIFJ1Y1wXPkuUakX0sikTA6sBPJ_Go-gTXP5tnxqX8ijskRRXVTI2UYSTeCse1-6pU6ikzFSw5KjdnZ9rDAfRXcLka5oAxwFTdPzbRqqQ7qgeWci7fWzWo8fa9d-KAiumiOHyJbFz6S0w83u9IAYqN1byR8wlXz_X0qkZ3mlb8duCYbmAmsRdBo4D0qDqNdEDyAMfAVsqHjF498MkZtheImcZpc8h1_4iSjXqNrQ")
		.await?;

	Ok(HttpResponse::Ok().finish())
}

///
/// Helper struct to create the routes and setup the service
///
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
