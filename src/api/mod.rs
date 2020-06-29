use crate::error::Error;
use crate::settings::Settings;
use crate::util::jwt::JsonValue;
use reqwest::Url;
use std::collections::HashMap;

pub struct Api {
	client: reqwest::Client,
	id_token_endpoint: Option<Url>,
}

///
/// API implementation
///
impl Api {
	/// Construct the API endpoints
	pub fn new(settings: &Settings) -> Result<Self, Error> {
		let mut id_token_endpoint: Option<Url> = None;
		if let Some(ref endpoint) = settings.api.id_token_endpoint {
			if !endpoint.is_empty() {
				id_token_endpoint = Some(Url::parse(endpoint)?);
			}
		}
		Ok(Self {
			client: reqwest::Client::new(),
			id_token_endpoint: id_token_endpoint,
		})
	}
	///
	/// When a new ID token is received
	///
	pub async fn on_id_token(&self, value: &JsonValue) -> Result<(), Error> {
		if let Some(ref endpoint) = self.id_token_endpoint {
			let mut map = HashMap::new();
			map.insert("id_token", value);
			let response = self
				.client
				.post(endpoint.as_str())
				.json(&map)
				.send()
				.await?;

			// If the response is invalid, does not let the user login
			let code = response.status().as_u16();
			if code < 200 || code >= 300 {
				return Err(Error::ApiError);
			}
		}
		Ok(())
	}
}
