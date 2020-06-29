use super::base::{Provider, TokenSet, Userinfo};
use super::oidc::ProviderOIDC;
use crate::error::Error;
use crate::settings::Settings;
use crate::util::jwt::JsonValue;
use std::time::SystemTime;

pub struct ProviderKeycloak {
	oidc: ProviderOIDC,
}

impl ProviderKeycloak {
	///
	/// Create a new keycloak provider
	///
	pub fn new(settings: &Settings) -> Result<Self, Error> {
		let oidc = ProviderOIDC::new(settings)?;
		Ok(Self { oidc: oidc })
	}
	///
	/// Normalize the token_set for keycloak providers
	///
	fn normalize_token_set(&self, token_set: Option<TokenSet>) -> Result<Option<TokenSet>, Error> {
		let mut token_set = token_set;
		if let Some(ref mut token_set) = token_set {
			if let Some(ref id_token) = token_set.id_token {
				if let Some(ref id_token_str) = id_token.as_str() {
					let id_token = self.jwt_decode(id_token_str);
					token_set.id_token = id_token;
				}
			}
		}
		Ok(token_set)
	}

	/// Decode a JWT sent by keycloak
	fn jwt_decode(&self, jwt: &str) -> Option<JsonValue> {
		let decoded = jsonwebtoken::dangerous_unsafe_decode::<JsonValue>(jwt);
		if decoded.is_err() {
			return None;
		}
		let claims = decoded.unwrap().claims;
		Some(claims)
	}
}

#[async_trait::async_trait]
impl Provider for ProviderKeycloak {
	///
	/// Same as oidc authorization url
	///
	fn get_authorization_url(&self, state: String) -> String {
		self.oidc.get_authorization_url(state)
	}
	///
	/// Same as oidc authorization url
	///
	fn get_logout_url(&self) -> String {
		self.oidc.get_logout_url()
	}
	///
	/// When using keycloak, the access_token itself contains the userinfo
	///
	async fn userinfo(&self, access_token: &str) -> Result<Option<Userinfo>, Error> {
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
	/// Grant the TokenSet code using the oidc, then normalize the token
	///
	async fn grant_authorization_code(&self, code: &str) -> Result<Option<TokenSet>, Error> {
		let token_set = self.oidc.grant_authorization_code(code).await?;
		self.normalize_token_set(token_set)
	}
	///
	/// Grant the TokenSet using the oidc, then normalize the token
	///
	async fn grant_password(
		&self,
		username: &str,
		password: &str,
	) -> Result<Option<TokenSet>, Error> {
		self.oidc.grant_password(username, password).await
	}
	///
	/// Get the OIDC authorization url
	///
	async fn grant_refresh_token(&self, refresh_token: &str) -> Result<Option<TokenSet>, Error> {
		let token_set = self.oidc.grant_refresh_token(refresh_token).await?;
		self.normalize_token_set(token_set)
	}
}
