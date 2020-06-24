use super::base::{Provider, TokenSet};
use crate::error::Error;
use crate::settings::Settings;
use reqwest::Url;

#[derive(Debug)]
pub struct ProviderOIDC {
	client: reqwest::Client,
	client_id: String,
	client_secret: String,
	auth_url: Url,
	token_url: Url,
	userinfo_url: Url,
	callback_url: Url,
}

impl ProviderOIDC {
	///
	/// Create a new OpenID Connect provider
	///
	pub fn new(settings: &Settings) -> Result<Self, Error> {
		let auth_url = Url::parse(&settings.provider.auth_url)
			.or_else(|_| Err(Error::SettingsError(String::from("invalid url"))))?;
		let token_url = Url::parse(&settings.provider.token_url)
			.or_else(|_| Err(Error::SettingsError(String::from("invalid url"))))?;
		let userinfo_url = Url::parse(&settings.provider.userinfo_url)
			.or_else(|_| Err(Error::SettingsError(String::from("invalid url"))))?;
		let callback_url = Url::parse(&settings.provider.callback_url)
			.or_else(|_| Err(Error::SettingsError(String::from("invalid url"))))?;

		Ok(Self {
			client: reqwest::Client::new(),
			client_id: settings.provider.client_id.clone(),
			client_secret: settings.provider.client_secret.clone(),
			auth_url: auth_url,
			token_url: token_url,
			userinfo_url: userinfo_url,
			callback_url: callback_url,
		})
	}
	///
	/// Grant a token
	///
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
		let body = res
			.unwrap()
			.json::<serde_json::Value>()
			.await
			.or_else(|_| Err(Error::RequestError))?;

		let access_token = body["access_token"].as_str();
		let refresh_token = body["refresh_token"].as_str();
		if access_token.is_none() || refresh_token.is_none() {
			return Ok(None);
		}
		Ok(Some(TokenSet {
			access_token: access_token.unwrap().to_owned(),
			refresh_token: refresh_token.unwrap().to_owned(),
			expires_in: body["expires_in"].as_i64(),
			id_token: None,
		}))
	}
}

#[async_trait::async_trait]
impl Provider for ProviderOIDC {
	///
	/// Get the OIDC authorization url
	///
	fn get_authorization_url(&self, state: String) -> String {
		let mut url = self.auth_url.clone();
		{
			let mut query_pairs = url.query_pairs_mut();
			query_pairs
				.append_pair("response_type", "code")
				.append_pair("scope", "openid email profile")
				.append_pair("client_id", &self.client_id)
				.append_pair("redirect_uri", self.callback_url.as_str());
			if !state.is_empty() {
				query_pairs.append_pair("state", &state);
			}
		}
		url.into_string()
	}
	///
	/// Request the userinfo
	///
	async fn userinfo(&self, access_token: &str) -> Result<Option<serde_json::Value>, Error> {
		let res = self
			.client
			.get(self.userinfo_url.as_str())
			.header("authorization", format!("bearer {}", access_token))
			.send()
			.await;

		if res.is_err() {
			let code = res.unwrap_err().status();
			if code.is_some() {
				let status_code = code.unwrap().as_u16();
				if status_code == 400 || status_code == 401 {
					return Ok(None);
				}
			}
			return Err(Error::RequestError);
		}

		let body = res.unwrap().json::<serde_json::Value>().await;
		if body.is_err() {
			return Err(Error::RequestError);
		}
		Ok(Some(body.unwrap()))
	}
	///
	/// Peform an authorization_code grant
	///
	async fn grant_authorization_code(&self, code: &str) -> Result<Option<TokenSet>, Error> {
		let params = [
			("grant_type", "authorization_code"),
			("client_id", &self.client_id),
			("client_secret", &self.client_secret),
			("redirect_uri", self.callback_url.as_str()),
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
			("client_id", &self.client_id),
			("client_secret", &self.client_secret),
			("refresh_token", refresh_token),
		];
		self.grant(&params).await
	}
}
