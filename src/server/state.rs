use crate::error::Error;
use crate::util::crypto::Crypto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
	pub url: Option<String>,
}

impl State {
	pub fn serialize_state(crypto: &Crypto, url: Option<String>) -> Result<String, Error> {
		let state = Self { url: url };
		let request_state_string =
			serde_json::to_string(&state).or_else(|_| Err(Error::CryptoCipherError))?;
		let request_state_string_encrypted = crypto.encrypt(&request_state_string)?;
		Ok(request_state_string_encrypted)
	}

	pub fn deserialize_state(crypto: &Crypto, token: &str) -> Result<Self, Error> {
		let token_decrypted = crypto.decrypt(token)?;
		let request_state: Self =
			serde_json::from_str(&token_decrypted).or_else(|_| Err(Error::CryptoCipherError))?;
		Ok(request_state)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_serialization() {
		let random = Crypto::create_random();
		let c = Crypto::new("Some key to test", random);

		let data = "Some random data";
		let state_token = State::serialize_state(&c, Some(String::from(data))).unwrap();
		let state = State::deserialize_state(&c, &state_token).unwrap();
		assert_eq!(data, state.url.unwrap());
	}
}
