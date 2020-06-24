mod args;
mod env;

use crate::error::Error;
use args::ArgsConfig;
use env::EnvironmentConfig;
use serde::Deserialize;

///
/// Settings
///
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
	pub listen: String,
	pub secret: String,
	pub jwt_secret: Option<String>,
	pub cookie: SettingsCookie,
	pub provider: SettingsProvider,
}

///
/// Settings for cookies
///
#[derive(Clone, Debug, Deserialize)]
pub struct SettingsCookie {
	pub access_token_name: String,
	pub refresh_token_name: String,
}

///
/// Settings for the provider
///
#[derive(Clone, Debug, Deserialize)]
pub struct SettingsProvider {
	pub provider: String,
	pub client_id: String,
	pub client_secret: String,
	pub auth_url: String,
	pub token_url: String,
	pub userinfo_url: String,
	pub callback_url: String,
}

impl Settings {
	pub fn new(rand: &dyn ring::rand::SecureRandom) -> Self {
		match Self::new_impl(rand) {
			Ok(s) => s,
			Err(e) => {
				log::error!("{}", e);
				ArgsConfig::show_help();
				std::process::exit(1)
			}
		}
	}

	fn new_impl(rand: &dyn ring::rand::SecureRandom) -> Result<Self, Error> {
		let mut s = config::Config::new();
		s.set_default("listen", "http://127.0.0.1:8088")?;
		s.set_default("cookie.access_token_name", "sat")?;
		s.set_default("cookie.refresh_token_name", "srt")?;
		s.set_default("provider.provider", "oidc")?;

		// Use args
		s.merge(ArgsConfig::new()?)?;

		// Check if needs to parse using the env
		if let Ok(prefix) = s.get_str("config.env") {
			s.merge(EnvironmentConfig::with_prefix(
				&prefix,
				&["cookie", "provider"],
			))?;
		}

		// If no secret is provided, use a random one
		if s.get_str("secret").is_err() {
			s.set("secret", generate_random_secret(rand, 32))?;
		}
		let settings: Self = s.try_into()?;
		Ok(settings)
	}
}

///
/// Generate a random secret
///
fn generate_random_secret(random: &dyn ring::rand::SecureRandom, size: usize) -> String {
	let mut bytes: Vec<u8> = Vec::with_capacity(size);
	bytes.resize(size, 0);
	random.fill(&mut bytes).unwrap();
	return base64::encode(bytes);
}
