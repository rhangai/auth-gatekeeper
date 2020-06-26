use crate::error::Error;
use jsonwebtoken::{EncodingKey, Header};

pub type JsonValue = serde_json::Value;

#[derive(Clone)]
pub struct JWT {
	encoding_key: Option<EncodingKey>,
}

///
/// API implementation
///
impl JWT {
	/// Construct the API endpoints
	pub fn new(secret: Option<String>) -> Result<Self, Error> {
		let mut encoding_key: Option<EncodingKey> = None;
		if let Some(ref secret) = secret {
			encoding_key = Some(jsonwebtoken::EncodingKey::from_secret(&secret.as_ref()));
		}
		Ok(Self {
			encoding_key: encoding_key,
		})
	}
	///
	/// Encode a json value
	///
	pub fn encode_value(&self, value: &JsonValue) -> Result<JsonValue, Error> {
		if let Some(ref encoding_key) = self.encoding_key {
			let encoded = jsonwebtoken::encode(&Header::default(), value, &encoding_key)?;
			Ok(serde_json::to_value(encoded)?)
		} else {
			Ok(value.clone())
		}
	}
	///
	/// Encode a json value
	///
	pub fn encode_str(&self, value: &JsonValue) -> Result<String, Error> {
		if let Some(ref encoding_key) = self.encoding_key {
			let encoded = jsonwebtoken::encode(&Header::default(), value, &encoding_key)?;
			Ok(encoded)
		} else {
			Ok(serde_json::to_string(value)?)
		}
	}
}
