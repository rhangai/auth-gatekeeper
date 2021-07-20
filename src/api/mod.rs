use crate::error::Error;
use crate::settings::Settings;
use crate::util::jwt::JsonValue;
use actix_web::{client::Client, cookie};
use serde::Serialize;
use std::collections::HashMap;
use url::Url;

pub struct Api {
	client: Client,
	id_token_endpoint: Option<Url>,
	logout_endpoint: Option<Url>,
}

///
/// API implementation
///
impl Api {
	/// Construct the API endpoints
	pub fn new(settings: &Settings) -> Result<Self, Error> {
		let id_token_endpoint = parse_url(&settings.api.id_token_endpoint)?;
		let logout_endpoint = parse_url(&settings.api.logout_endpoint)?;

		Ok(Self {
			client: Client::new(),
			id_token_endpoint: id_token_endpoint,
			logout_endpoint: logout_endpoint,
		})
	}
	///
	/// When a new ID token is received
	///
	pub async fn on_id_token<'a>(
		&self,
		cookies: &'a mut Option<Vec<cookie::Cookie<'static>>>,
		value: &JsonValue,
	) -> Result<(), Error> {
		self.request_endpoint(cookies, &self.id_token_endpoint, || {
			let mut map = HashMap::new();
			map.insert("id_token", value);
			Some(map)
		})
		.await
	}
	///
	/// Perform a logout request
	///
	pub async fn on_logout<'a>(
		&self,
		cookies: &'a mut Option<Vec<cookie::Cookie<'static>>>,
	) -> Result<(), Error> {
		self.request_endpoint(cookies, &self.logout_endpoint, || None as Option<()>)
			.await
	}

	async fn request_endpoint<'a, T, F>(
		&self,
		cookies: &'a mut Option<Vec<cookie::Cookie<'static>>>,
		endpoint: &Option<Url>,
		data_fn: F,
	) -> Result<(), Error>
	where
		T: Serialize + std::fmt::Debug,
		F: std::ops::FnOnce() -> Option<T>,
	{
		if let Some(ref endpoint) = endpoint {
			let request = self.client.post(endpoint.as_str());
			let data = data_fn();
			let response = if let Some(data) = data {
				request.send_json(&data).await?
			} else {
				request.send().await?
			};

			// If the response is invalid, does not let the user login
			let code = response.status().as_u16();
			if code < 200 || code >= 300 {
				return Err(Error::ApiError);
			}

			if let Some(ref mut cookies) = cookies {
				let set_cookie = response.headers().get_all("set-cookie");
				for cookie_value in set_cookie {
					if let Ok(cookie_str) = cookie_value.to_str() {
						let cookie = cookie::Cookie::parse(cookie_str);
						if cookie.is_err() {
							continue;
						}
						let cookie = cookie.unwrap();
						cookies.push(cookie.into_owned());
					}
				}
			}
		}
		Ok(())
	}
}

fn parse_url(url: &Option<String>) -> Result<Option<Url>, Error> {
	if let Some(ref endpoint) = url {
		if !endpoint.is_empty() {
			return Ok(Some(Url::parse(endpoint)?));
		}
	}
	Ok(None)
}
