use crate::util::crypto::RandomPtr;
use envconfig::Envconfig;

#[derive(Debug, Clone, Envconfig)]
pub struct Config {
	#[envconfig(from = "AUTH_GATEKEEPER_LISTEN", default = "http://127.0.0.1:8088")]
	pub listen: String,
	#[envconfig(from = "AUTH_GATEKEEPER_SECRET")]
	pub secret: Option<String>,
	#[envconfig(from = "AUTH_GATEKEEPER_JWT_SECRET")]
	pub jwt_secret: Option<String>,
	#[envconfig(from = "AUTH_GATEKEEPER_COOKIE_ACCESS_TOKEN_NAME", default = "sat")]
	pub cookie_access_token_name: String,
	#[envconfig(from = "AUTH_GATEKEEPER_COOKIE_REFRESH_TOKEN_NAME", default = "srt")]
	pub cookie_refresh_token_name: String,
	#[envconfig(from = "AUTH_GATEKEEPER_PROVIDER", default = "oidc")]
	pub provider: String,
	#[envconfig(from = "AUTH_GATEKEEPER_PROVIDER_CLIENT_ID")]
	pub provider_client_id: String,
	#[envconfig(from = "AUTH_GATEKEEPER_PROVIDER_CLIENT_SECRET")]
	pub provider_client_secret: String,
	#[envconfig(from = "AUTH_GATEKEEPER_PROVIDER_AUTH_URL")]
	pub provider_auth_url: String,
	#[envconfig(from = "AUTH_GATEKEEPER_PROVIDER_TOKEN_URL")]
	pub provider_token_url: String,
	#[envconfig(from = "AUTH_GATEKEEPER_PROVIDER_USERINFO_URL")]
	pub provider_userinfo_url: String,
	#[envconfig(from = "AUTH_GATEKEEPER_PROVIDER_CALLBACK_URL")]
	pub provider_callback_url: String,
	// pub provider_jwks_url: String,
	// pub api: String,
	// pub api_authorization: String,
	// pub api_id_token_endpoint: String,
}

impl Config {
	pub fn parse(random: RandomPtr) -> Config {
		let mut config = Config::init().unwrap();
		if config.secret.is_none() {
			config.secret = Some(generate_random_secret(&random, 32));
		}
		config
	}
}

fn generate_random_secret(random: &RandomPtr, size: usize) -> String {
	let mut bytes: Vec<u8> = Vec::with_capacity(size);
	bytes.resize(size, 0);
	random.fill(&mut bytes).unwrap();
	return base64::encode(bytes);
}
