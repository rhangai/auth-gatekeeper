use super::base::{Provider, TokenSet, Userinfo};
use crate::error::Error;
use crate::settings::Settings;
use crate::util::jwt::JsonValue;
use actix_web::{client::ClientBuilder, ResponseError};
use std::time::SystemTime;
use url::Url;

pub struct ProviderOIDCOptions {
	pub userinfo_from_access_token: bool,
}

pub struct ProviderOIDC {
	client_id: String,
	client_secret: String,
	scope: String,
	auth_url: Url,
	token_url: Url,
	userinfo_url: Url,
	end_session_url: Option<Url>,
	callback_url: Url,
	logout_redirect_url: Url,
	options: ProviderOIDCOptions,
}

impl ProviderOIDC {
	///
	/// Create a new OpenID Connect provider
	///
	pub fn new(settings: &Settings, options: ProviderOIDCOptions) -> Result<Self, Error> {
		let auth_url = Url::parse(&settings.provider.auth_url)?;
		let token_url = Url::parse(&settings.provider.token_url)?;
		let userinfo_url = Url::parse(&settings.provider.userinfo_url)?;
		let callback_url = Url::parse(&settings.provider.callback_url)?;
		let logout_redirect_url = Url::parse(&settings.provider.logout_redirect_url)?;
		let end_session_url = if let Some(ref url) = &settings.provider.end_session_url {
			Some(url.parse::<Url>()?)
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
			options,
		})
	}
	///
	/// Get the id_token from the grant
	///
	fn get_id_token(&self, obj: &serde_json::Value) -> Option<JsonValue> {
		let id_token = obj.get("id_token");
		if id_token.is_none() {
			return None;
		}
		let id_token = id_token.unwrap();
		if let serde_json::Value::Null = id_token {
			return None;
		}
		if let Some(ref id_token_str) = id_token.as_str() {
			let decoded = self.jwt_decode(id_token_str);
			if decoded.is_some() {
				return decoded;
			}
		}
		Some(id_token.clone())
	}

	/// Decode a JWT sent by keycloak
	fn jwt_decode(&self, jwt: &str) -> Option<JsonValue> {
		let decoded = jsonwebtoken::dangerous_insecure_decode::<JsonValue>(jwt);
		if decoded.is_err() {
			return None;
		}
		let claims = decoded.unwrap().claims;
		Some(claims)
	}

	///
	/// Grant a token
	///
	async fn grant<T: serde::Serialize + ?Sized>(
		&self,
		form: &T,
	) -> Result<Option<TokenSet>, Error> {
		let client = ClientBuilder::new().timeout(std::time::Duration::new(30, 0)).finish();
		let mut res = client
			.post(self.token_url.as_str())
			.send_form(&form)
			.await?;
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
			id_token: self.get_id_token(&body),
		}))
	}

	///
	/// Request the userinfo
	///
	pub async fn userinfo(&self, access_token: &str) -> Result<Option<Userinfo>, Error> {
		if self.options.userinfo_from_access_token {
			return self.get_userinfo_from_access_token(access_token).await;
		}
		self.get_userinfo_from_oidc(access_token).await
	}
	///
	/// Request the userinfo using openid client
	///
	async fn get_userinfo_from_oidc(&self, access_token: &str) -> Result<Option<Userinfo>, Error> {
		let client = ClientBuilder::new().timeout(std::time::Duration::new(30, 0)).finish();
		let res = client
			.get(self.userinfo_url.as_str())
			.header("authorization", format!("Bearer {}", access_token))
			.send();

		let res = res.await;
		if res.is_err() {
			let error = res.unwrap_err();
			let code = error.status_code();
			if code == 400 || code == 401 {
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
	/// Request the userinfo
	///
	async fn get_userinfo_from_access_token(
		&self,
		access_token: &str,
	) -> Result<Option<Userinfo>, Error> {
		let claims = self.jwt_decode(access_token);
		if claims.is_none() {
			return Ok(None);
		}
		let claims = claims.unwrap();

		let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);

		let exp = claims["exp"].as_u64();
		if exp.is_some() && exp.unwrap() <= now.unwrap().as_secs() {
			return Ok(None);
		}
		Ok(Some(Userinfo {
			data: claims,
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
			("scope", &self.scope),
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

impl Provider for ProviderOIDC {
	///
	/// Get the OIDC authorization url
	///
	fn get_authorization_url(&self, state: String) -> String {
		let mut auth_url = self.auth_url.clone();
		{
			let mut query = auth_url.query_pairs_mut();
			query
				.append_pair("response_type", "code")
				.append_pair("scope", &self.scope)
				.append_pair("client_id", &self.client_id)
				.append_pair("redirect_uri", self.callback_url.as_str());
			if !state.is_empty() {
				query.append_pair("state", &state);
			}
		}
		auth_url.to_string()
	}
	///
	/// Get the OIDC logout url
	///
	fn get_logout_url(&self) -> String {
		let logout_url = self.logout_redirect_url.to_string();
		if self.end_session_url.is_none() {
			return logout_url;
		}

		let mut end_session_url = self.end_session_url.clone().unwrap();
		{
			let mut query = end_session_url.query_pairs_mut();
			query
				.append_pair("client_id", &self.client_id)
				.append_pair("redirect_uri", &logout_url);
		}
		end_session_url.to_string()
	}
}
