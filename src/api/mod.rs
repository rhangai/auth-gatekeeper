use crate::error::Error;
use crate::settings::Settings;
use jsonwebtoken::{EncodingKey, Header};
use reqwest::Url;
use std::collections::HashMap;

pub struct Api {
	client: reqwest::Client,
	jwt_encoding_key: Option<EncodingKey>,
	id_token_endpoint: Option<Url>,
}

impl Api {
	pub fn new(settings: &Settings) -> Result<Self, Error> {
		let mut id_token_endpoint: Option<Url> = None;
		if let Some(ref endpoint) = settings.api.id_token_endpoint {
			id_token_endpoint = Some(Url::parse(endpoint)?);
		}

		let mut jwt_encoding_key: Option<EncodingKey> = None;
		if let Some(ref jwt_secret) = settings.jwt_secret {
			jwt_encoding_key = Some(jsonwebtoken::EncodingKey::from_secret(&jwt_secret.as_ref()));
		}
		Ok(Self {
			client: reqwest::Client::new(),
			jwt_encoding_key: jwt_encoding_key,
			id_token_endpoint: id_token_endpoint,
		})
	}
	///
	/// When a new ID token is received
	///
	pub async fn on_id_token(&self, value: &serde_json::Value) -> Result<(), Error> {
		if self.id_token_endpoint.is_none() {
			return Ok(());
		}

		let endpoint = self.id_token_endpoint.as_ref().unwrap();
		let data: serde_json::Value;

		// If there is an encoding key, then jwt encodes it
		if let Some(ref encoding_key) = self.jwt_encoding_key {
			let id_token = jsonwebtoken::encode(&Header::default(), value, &encoding_key)?;

			let mut map = HashMap::new();
			map.insert("id_token", id_token);
			data = serde_json::to_value(map)?;
		} else {
			let mut map = HashMap::new();
			map.insert("id_token", value);
			data = serde_json::to_value(map)?;
		}

		// Perform a request to the endpoint
		self.client
			.post(endpoint.as_str())
			.json(&data)
			.send()
			.await?;
		Ok(())
	}
}
