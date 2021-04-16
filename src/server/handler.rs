use super::data::Data;
use super::state::State;
use crate::error::Error;
use crate::session::{Session, SessionFlags};
use crate::settings::Settings;
use crate::util::crypto;
use crate::util::jwt::JsonValue;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

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

#[derive(Deserialize)]
struct AuthLoginQuery {
	url: Option<String>,
}

#[derive(Deserialize)]
struct AuthLoginForm {
	username: String,
	password: String,
}

#[derive(Deserialize)]
struct AuthForwardAuthQuery {
	redirect: Option<String>,
}

///
/// Redirect to the login url using authorization_code flow
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
/// Post to the login page using password flow
///
async fn route_post_login(
	data: web::Data<Data>,
	req: HttpRequest,
	query: web::Query<AuthLoginQuery>,
	form: web::Form<AuthLoginForm>,
) -> Result<impl Responder, Error> {
	// Perform the grant
	let token_set = data
		.provider
		.grant_password(&form.username, &form.password)
		.await?;
	if token_set.is_none() {
		return Ok(HttpResponse::Unauthorized().finish());
	}

	// Create the response and redirects
	let mut builder = HttpResponse::Found();
	let session = Session::new(data.clone(), token_set.unwrap());
	session
		.response(&req, &mut builder, SessionFlags::COOKIES)
		.await?;
	if let Some(ref url) = query.url {
		builder.header("location", url.clone());
	} else {
		builder.header("location", "/");
	}
	Ok(builder.finish())
}

///
/// Logout
///
async fn route_logout(data: web::Data<Data>, req: HttpRequest) -> Result<impl Responder, Error> {
	let url = data.provider.get_logout_url();
	let session = Session::logout(data);
	let mut builder = HttpResponse::Found();
	builder.header("location", url);
	session
		.response(&req, &mut builder, SessionFlags::COOKIES)
		.await?;
	Ok(builder.finish())
}

///
/// Callback for the authorization_code grant
///
async fn route_callback(
	data: web::Data<Data>,
	req: HttpRequest,
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
	let session = Session::new(data.clone(), token_set.unwrap());
	session
		.response(&req, &mut builder, SessionFlags::COOKIES)
		.await?;
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
async fn route_refresh(data: web::Data<Data>, req: HttpRequest) -> Result<impl Responder, Error> {
	let mut session = Session::from_request(data, &req);
	session.validate(true).await?;

	let mut builder = HttpResponse::Ok();
	session
		.response(&req, &mut builder, SessionFlags::COOKIES)
		.await?;

	let userinfo = session.get_userinfo();
	if let Some(userinfo) = userinfo {
		let mut data: HashMap<&str, &JsonValue> = HashMap::new();
		if let Some(ref user_sub) = userinfo.data.get("sub") {
			data.insert("sub", user_sub);
		}
		if let Some(ref user_email) = userinfo.data.get("email") {
			data.insert("email", user_email);
		}
		if let Some(ref user_name) = userinfo.data.get("name") {
			data.insert("name", user_name);
		}
		if let Some(ref user_realm_access) = userinfo.data.get("realm_access") {
			if let Some(ref user_roles) = user_realm_access.get("roles") {
				data.insert("roles", user_roles);
			}
		}
		Ok(builder.json(data))
	} else {
		Ok(builder.finish())
	}
}

///
/// Validate the login
///
async fn route_validate(data: web::Data<Data>, req: HttpRequest) -> Result<impl Responder, Error> {
	let mut session = Session::from_request(data, &req);
	session.validate(true).await?;

	let mut builder = HttpResponse::Ok();
	session
		.response(&req, &mut builder, SessionFlags::X_AUTH_HEADERS)
		.await?;
	Ok(builder.finish())
}

///
/// Endpoint middleware for traefik
///
async fn route_forward_auth(
	data: web::Data<Data>,
	req: HttpRequest,
	query: web::Query<AuthForwardAuthQuery>,
) -> Result<impl Responder, Error> {
	let mut session = Session::from_request(data, &req);
	session.validate(true).await?;

	let mut builder = HttpResponse::Ok();
	let flags = if query.redirect.is_some() {
		SessionFlags::FORWARD_AUTH | SessionFlags::FORWARD_AUTH_REDIRECT
	} else {
		SessionFlags::FORWARD_AUTH
	};
	session.response(&req, &mut builder, flags).await?;
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
			.route("/login", web::post().to(route_post_login))
			.route("/logout", web::get().to(route_logout))
			.route("/auth/callback", web::get().to(route_callback))
			.route("/auth/refresh", web::get().to(route_refresh))
			.route("/auth/validate", web::get().to(route_validate))
			.route("/auth/forward-auth", web::get().to(route_forward_auth));
		Ok(())
	}
}
