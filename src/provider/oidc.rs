use super::base::{Provider, TokenSet};
use crate::config::Config;
use crate::error::Error;
use reqwest::Url;

#[derive(Debug)]
pub struct ProviderOIDC {
	client: reqwest::Client,
	auth_url: Url,
	token_url: Url,
}

impl ProviderOIDC {
	pub fn new(config: &Config) -> Result<Self, Error> {
		let auth_url =
			Url::parse(&config.provider_auth_url).or_else(|_| Err(Error::ConfigError))?;

		let token_url =
			Url::parse(&config.provider_token_url).or_else(|_| Err(Error::ConfigError))?;

		Ok(Self {
			client: reqwest::Client::new(),
			auth_url: auth_url,
			token_url: token_url,
		})
	}

	async fn grant<T: serde::Serialize + ?Sized>(
		&self,
		form: &T,
	) -> Result<Option<TokenSet>, Error> {
		let res = self
			.client
			.post(self.token_url.as_str())
			.form(form)
			.send()
			.await;
		let body = res.unwrap().json::<serde_json::Value>().await;
		Ok(Some(TokenSet {
			access_token: String::from("oi"),
			refresh_token: String::from("oi"),
		}))
	}
}

#[async_trait::async_trait]
impl Provider for ProviderOIDC {
	fn get_authorization_url(&self, state: String) -> String {
		let mut url = self.auth_url.clone();
		let mut query_pairs = url.query_pairs_mut();
		query_pairs
			.append_pair("response_type", "code")
			.append_pair("scope", "openid email profile")
			.append_pair("client_id", "");
		if !state.is_empty() {
			query_pairs.append_pair("state", &state);
		}
		drop(query_pairs);
		url.into_string()
	}

	async fn userinfo(&self, access_token: &str) -> Result<Option<serde_json::Value>, Error> {
		Ok(None)
	}

	///
	/// Peform an authorization_code grant
	///
	async fn grant_authorization_code(&self, code: &str) -> Result<Option<TokenSet>, Error> {
		let params = [
			("grant_type", "authorization_code"),
			("client_id", "teste"),
			("client_secret", "teste"),
			("redirect_uri", "teste"),
			("code", code),
		];
		self.grant(&params).await
	}

	///
	/// Peform a refresh_token grant
	///
	async fn grant_refresh_token(&self, refresh_token: &str) -> Result<Option<TokenSet>, Error> {
		let params = [
			("grant_type", "refresh_token"),
			("client_id", "teste"),
			("client_secret", "teste"),
			("refresh_token", refresh_token),
		];
		self.grant(&params).await
	}
}
