use super::base::{Provider, TokenSet, Userinfo};
use crate::error::Error;
use crate::settings::Settings;
use actix_web::{client::Client, http::Uri, ResponseError};

pub struct ProviderOIDC {
	client_id: String,
	client_secret: String,
	scope: String,
	auth_url: Uri,
	token_url: Uri,
	userinfo_url: Uri,
	end_session_url: Option<Uri>,
	callback_url: Uri,
	logout_redirect_url: Uri,
}

impl ProviderOIDC {
	///
	/// Create a new OpenID Connect provider
	///
	pub fn new(settings: &Settings) -> Result<Self, Error> {
		let auth_url = settings.provider.auth_url.parse::<Uri>()?;
		let token_url = settings.provider.token_url.parse::<Uri>()?;
		let userinfo_url = settings.provider.userinfo_url.parse::<Uri>()?;
		let callback_url = settings.provider.callback_url.parse::<Uri>()?;
		let logout_redirect_url = settings.provider.logout_redirect_url.parse::<Uri>()?;
		let end_session_url = if let Some(ref url) = &settings.provider.end_session_url {
			Some(url.parse::<Uri>()?)
		} else {
			None
		};

		Ok(Self {
			client_id: settings.provider.client_id.clone(),
			client_secret: settings.provider.client_secret.clone(),
			scope: settings
				.provider
				.scope
				.clone()
				.unwrap_or_else(|| String::from("openid email profile offline_access")),
			auth_url: auth_url,
			token_url: token_url,
			userinfo_url: userinfo_url,
			end_session_url: end_session_url,
			callback_url: callback_url,
			logout_redirect_url,
		})
	}
	///
	/// Grant a token
	///
	async fn grant<T: serde::Serialize + ?Sized>(
		&self,
		form: &T,
	) -> Result<Option<TokenSet>, Error> {
		let client = Client::new();
		let mut res = client.post(&self.token_url).send_form(&form).await?;
		let body = res.json::<serde_json::Value>().await?;

		let access_token = body["access_token"].as_str();
		let refresh_token = body["refresh_token"].as_str();
		if access_token.is_none() || refresh_token.is_none() {
			return Ok(None);
		}
		Ok(Some(TokenSet {
			access_token: access_token.unwrap().to_owned(),
			refresh_token: refresh_token.unwrap().to_owned(),
			expires_in: body["expires_in"].as_i64(),
			id_token: to_value(&body, "id_token"),
		}))
	}

	///
	/// Request the userinfo
	///
	pub async fn userinfo(&self, access_token: &str) -> Result<Option<Userinfo>, Error> {
		let client = Client::new();
		let res = client
			.get(self.userinfo_url.clone())
			.header("authorization", format!("Bearer {}", access_token))
			.send();

		let res = res.await;
		if res.is_err() {
			let error = res.unwrap_err();
			let code = error.status_code();
			if code == 400 {
				return Ok(None);
			}
			return Err(Error::RequestError(error));
		}

		let body = res.unwrap().json::<serde_json::Value>().await?;
		Ok(Some(Userinfo {
			data: body,
			expires_at: None,
		}))
	}
	///
	/// Peform an authorization_code grant
	///
	pub async fn grant_authorization_code(&self, code: &str) -> Result<Option<TokenSet>, Error> {
		let callback = self.callback_url.to_string();
		let params = [
			("grant_type", "authorization_code"),
			("client_id", &self.client_id),
			("client_secret", &self.client_secret),
			("redirect_uri", &callback),
			("code", code),
		];
		self.grant(&params).await
	}
	///
	/// Perform a password grant
	///
	pub async fn grant_password(
		&self,
		username: &str,
		password: &str,
	) -> Result<Option<TokenSet>, Error> {
		let params = [
			("grant_type", "password"),
			("client_id", &self.client_id),
			("client_secret", &self.client_secret),
			("username", username),
			("password", password),
		];
		self.grant(&params).await
	}
	///
	/// Peform a refresh_token grant
	///
	pub async fn grant_refresh_token(
		&self,
		refresh_token: &str,
	) -> Result<Option<TokenSet>, Error> {
		let params = [
			("grant_type", "refresh_token"),
			("client_id", &self.client_id),
			("client_secret", &self.client_secret),
			("refresh_token", refresh_token),
		];
		self.grant(&params).await
	}
}

fn to_value(obj: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
	let value = obj.get(key);
	if value.is_none() {
		return None;
	}
	let value = value.unwrap();
	if let serde_json::Value::Null = value {
		return None;
	}
	Some(value.clone())
}

impl Provider for ProviderOIDC {
	///
	/// Get the OIDC authorization url
	///
	fn get_authorization_url(&self, state: String) -> String {
		let mut builder = Uri::builder();

		builder = builder.scheme(self.auth_url.scheme_str().unwrap_or("http"));
		if let Some(authority) = self.auth_url.authority() {
			builder = builder.authority(authority.clone());
		}

		{
			let mut query = qstring::QString::from(self.auth_url.query().unwrap_or(""));
			query.add_pair(("response_type", "code"));
			query.add_pair(("scope", &self.scope));
			query.add_pair(("client_id", &self.client_id));
			query.add_pair(("redirect_uri", self.callback_url.to_string()));
			if !state.is_empty() {
				query.add_pair(("state", &state));
			}
			let path_and_query = format!("{}?{}", self.auth_url.path(), query.to_string());
			builder = builder.path_and_query(path_and_query);
		}
		builder.build().unwrap().to_string()
	}
	///
	/// Get the OIDC logout url
	///
	fn get_logout_url(&self) -> String {
		let logout_url = self.logout_redirect_url.to_string();
		if self.end_session_url.is_none() {
			return logout_url;
		}

		let mut builder = Uri::builder();
		let end_session_url = self.end_session_url.clone().unwrap();

		builder = builder.scheme(end_session_url.scheme_str().unwrap_or("http"));
		if let Some(authority) = end_session_url.authority() {
			builder = builder.authority(authority.clone());
		}

		{
			let mut query = qstring::QString::from(end_session_url.query().unwrap_or(""));
			query.add_pair(("redirect_uri", logout_url));
			let path_and_query = format!("{}?{}", end_session_url.path(), query.to_string());
			builder = builder.path_and_query(path_and_query);
		}
		builder.build().unwrap().to_string()
	}
}
