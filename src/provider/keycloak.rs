use super::base::{Provider, TokenSet};
use super::oidc::ProviderOIDC;
use crate::error::Error;
use crate::settings::Settings;
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
}

#[async_trait::async_trait]
impl Provider for ProviderKeycloak {
	///
	/// Get the OIDC authorization url
	///
	fn get_authorization_url(&self, state: String) -> String {
		self.oidc.get_authorization_url(state)
	}
	///
	/// When using keycloak, the access_token itself contains the userinfo
	///
	async fn userinfo(&self, access_token: &str) -> Result<Option<serde_json::Value>, Error> {
		let userinfo = jsonwebtoken::dangerous_unsafe_decode::<serde_json::Value>(access_token);
		if userinfo.is_err() {
			return Ok(None);
		}

		let claims = userinfo.unwrap().claims;

		let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);

		let exp = claims["exp"].as_u64();
		if exp.is_some() && exp.unwrap() <= now.unwrap().as_secs() {
			return Ok(None);
		}
		Ok(Some(claims))
	}
	///
	/// Get the OIDC authorization url
	///
	async fn grant_authorization_code(&self, code: &str) -> Result<Option<TokenSet>, Error> {
		self.oidc.grant_authorization_code(code).await
	}
	///
	/// Get the OIDC authorization url
	///
	async fn grant_refresh_token(&self, refresh_token: &str) -> Result<Option<TokenSet>, Error> {
		self.oidc.grant_refresh_token(refresh_token).await
	}
}
