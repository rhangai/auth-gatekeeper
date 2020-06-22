#[derive(Clone)]
pub struct Config {
	pub cookie_secret: String,
	pub cookie_access_token_name: String,
	pub cookie_refresh_token_name: String,
	// pub provider: String,
	pub provider_client_id: String,
	pub provider_client_secret: String,
	pub provider_auth_url: String,
	pub provider_token_url: String,
	// pub provider_userinfo_url: String,
	// pub provider_callback_url: String,
	// pub provider_jwks_url: String,
	// pub api: String,
	// pub api_authorization: String,
	// pub api_id_token_endpoint: String,
}

impl Config {
	pub fn new() -> Config {
		Config {
			cookie_secret: String::from("oi"),
			cookie_access_token_name: String::from("sat"),
			cookie_refresh_token_name: String::from("srt"),
			provider_client_id: String::from("ok"),
			provider_client_secret: String::from("ok"),
			provider_auth_url: String::from("ok"),
			provider_token_url: String::from("ok"),
		}
	}
}
