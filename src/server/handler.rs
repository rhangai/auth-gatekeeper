use super::data::Data;
use super::state::State;
use crate::error::Error;
use crate::session::{Session, SessionFlags};
use crate::settings::Settings;
use crate::util::crypto;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginQuery {
	state: Option<String>,
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
async fn route_login(
	data: web::Data<Data>,
	query: web::Query<LoginQuery>,
) -> Result<impl Responder, Error> {
	let state_str = if let Some(ref state) = query.state {
		state.clone()
	} else {
		State::serialize_state(&data.crypto, query.url.clone())?
	};
	let url = data.provider.get_authorization_url(state_str);
	let response = HttpResponse::Found().header("location", url).finish();
	Ok(response)
}

///
/// Callback
///
async fn route_callback(
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
	let session = Session::new(data.clone(), token_set.unwrap());
	let mut builder = HttpResponse::Found();
	session.response(&mut builder, SessionFlags::COOKIES);
	{
		let mut location: String = String::from("/");
		if query.state.is_some() {
			let state = State::deserialize_state(&data.crypto, &query.state.as_ref().unwrap());
			if state.is_ok() {
				location = state.unwrap().url.unwrap_or(location);
			}
		}
		if location.is_empty() {
			location = String::from("/");
		}
		builder.header("location", location);
	}
	Ok(builder.finish())
}

///
/// Validate the login
///
async fn route_validate(data: web::Data<Data>, req: HttpRequest) -> Result<impl Responder, Error> {
	let mut session = Session::from_request(data, &req);
	session.validate().await?;

	let mut builder = HttpResponse::Ok();
	session.response(&mut builder, SessionFlags::X_HEADERS);
	Ok(builder.finish())
}

///
/// Helper struct to create the routes and setup the service
///
pub struct Handler {
	random: crypto::RandomPtr,
	settings: Settings,
}
impl Handler {
	///
	/// Create a new handler
	///
	pub fn new(random: crypto::RandomPtr, settings: Settings) -> Result<Handler, Error> {
		Ok(Handler {
			random: random,
			settings: settings,
		})
	}

	///
	/// Configure the service
	///
	pub fn config(&self, service_config: &mut web::ServiceConfig) -> Result<(), Error> {
		let data = Data::new(self.settings.clone(), self.random.clone())?;
		service_config
			.data(data)
			.route("/login", web::get().to(route_login))
			.route("/auth/callback", web::get().to(route_callback))
			.route("/auth/validate", web::get().to(route_validate));
		Ok(())
	}
}
