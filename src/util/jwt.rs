use crate::error::Error;
use jsonwebtoken::{EncodingKey, Header};
use serde::ser::Serialize;

pub type JsonValue = serde_json::Value;

#[derive(Clone)]
pub struct JWT {
	encoding_key: Option<EncodingKey>,
}

///
/// JWT
///
impl JWT {
	/// Construct the API endpoints
	pub fn new<T>(secret: Option<T>) -> Result<Self, Error>
	where
		T: Into<String>,
	{
		let mut encoding_key: Option<EncodingKey> = None;
		if let Some(secret) = secret {
			let secret_str: String = secret.into();
			encoding_key = Some(jsonwebtoken::EncodingKey::from_secret(&secret_str.as_ref()));
		}
		Ok(Self {
			encoding_key: encoding_key,
		})
	}
	///
	/// Encode a json value
	///
	pub fn encode_value<T>(&self, value: &T) -> Result<JsonValue, Error>
	where
		T: Serialize,
	{
		if let Some(ref encoding_key) = self.encoding_key {
			let encoded = jsonwebtoken::encode(&Header::default(), value, &encoding_key)?;
			Ok(serde_json::to_value(encoded)?)
		} else {
			Ok(serde_json::to_value(value.clone())?)
		}
	}
	///
	/// Encode something into a string
	///
	pub fn encode_str<T>(&self, value: &T) -> Result<String, Error>
	where
		T: Serialize,
	{
		if let Some(ref encoding_key) = self.encoding_key {
			let encoded = jsonwebtoken::encode(&Header::default(), value, &encoding_key)?;
			Ok(encoded)
		} else {
			Ok(serde_json::to_string(value)?)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_encryption() {
		let secret = "my secret";
		let jwt = JWT::new(Some(secret)).unwrap();

		let mut data = std::collections::HashMap::new();
		data.insert("jet", String::from("áéíóú"));
		let encoded = jwt.encode_str(&data).unwrap();
		println!("{:?}", data);
		println!("{}", encoded);
		// assert_eq!("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.InNvbWV0aGluZyI.9T-IJnr5l7oe5yWKhI8T95Iz1Ju8qPJEhAqOiIjab_w", encoded);
	}
}
