use crate::error::Error;
use config::{Source, Value};
use getopts::Options;
use std::collections::HashMap;

const OPTS: &[(&str, &str, &str, &str)] = &[
	(
		"config-env",
		"config.env",
		"Use env variables prefixed with ENV_PREFIX to configure. Ex: '--listen' becomes ENV_PREFIX_LISTEN",
		"ENV_PREFIX",
	),
	(
		"listen",
		"listen",
		"Listen for the server on the given URLs (comma delimited for multiple)",
		"URLS",
	),
	(
		"secret",
		"secret",
		"The SECRET is used to encrypt the cookies",
		"SECRET",
	),
	(
		"jwt-secret",
		"jwt_secret",
		"Use SECRET to encode the JWT used by x-auth headers",
		"SECRET",
	),
	(
		"data",
		"data",
		"Arbitrary DATA to pass directly to x-auth-data header",
		"DATA",
	),
	(
		"api-id-token-endpoint",
		"api.id_token_endpoint",
		"The endpoint to call everytime a new id_token is found",
		"ENDPOINT",
	),
	(
		"cookie-access-token-name",
		"cookie.access_token_name",
		"The name of the cookie used to store the access token",
		"NAME",
	),
	(
		"cookie-refresh-token-name",
		"cookie.refresh_token_name",
		"The name of the cookie used to store the refresh token",
		"NAME",
	),
	(
		"provider",
		"provider.provider",
		"The provider to use. 'oidc' or 'keycloak'",
		"PROVIDER",
	),
	(
		"provider-client-id",
		"provider.client_id",
		"Client ID of the provider",
		"CLIENT_ID",
	),
	(
		"provider-client-secret",
		"provider.client_secret",
		"Client Secret of the provider",
		"CLIENT_SECRET",
	),
	(
		"provider-auth-url",
		"provider.auth_url",
		"Url of the authorization endpoint",
		"URL",
	),
	(
		"provider-token-url",
		"provider.token_url",
		"Url of the token endpoint",
		"URL",
	),
	(
		"provider-userinfo-url",
		"provider.userinfo_url",
		"Url to get the user info using the access token",
		"URL",
	),
	(
		"provider-end-session-url",
		"provider.end_session_url",
		"Url to logout",
		"URL",
	),
	(
		"provider-logout-redirect-url",
		"provider.logout_redirect_url",
		"Url to send the user after the logout",
		"URL",
	),
	(
		"provider-callback-url",
		"provider.callback_url",
		"Url to send the user back when auth is complete",
		"URL",
	),
];

#[derive(Clone, Debug)]
pub struct ArgsConfig {
	config: HashMap<String, Value>,
}

impl ArgsConfig {
	pub fn new() -> Result<Self, Error> {
		Ok(Self {
			config: Self::collect_config()?,
		})
	}

	pub fn show_help() {
		let options = Self::get_options();
		let brief = format!("Usage: auth-gatekeeper [options]");
		print!("{}", options.usage(&brief));
	}

	fn get_options() -> Options {
		let mut opts = Options::new();
		opts.optflag("h", "help", "Show help");
		for (longname, _, desc, arg_hint) in OPTS {
			opts.optopt("", longname, desc, arg_hint);
		}
		opts
	}

	fn collect_config() -> Result<HashMap<String, Value>, Error> {
		let uri = String::from("args");
		let opts = Self::get_options();
		let args: Vec<String> = std::env::args().collect();
		let matches = opts.parse(&args[1..]).unwrap();

		if matches.opt_present("h") {
			return Err(Error::SettingsShowHelpError);
		}

		let mut m: HashMap<String, Value> = HashMap::new();
		for (longname, configname, _, _) in OPTS {
			if let Some(value) = matches.opt_str(longname) {
				m.insert(configname.to_string(), Value::new(Some(&uri), value));
			}
		}
		Ok(m)
	}
}

///
/// Implementaton
impl Source for ArgsConfig {
	fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
		Box::new((*self).clone())
	}

	fn collect(&self) -> Result<HashMap<String, Value>, config::ConfigError> {
		Ok(self.config.clone())
	}
}
