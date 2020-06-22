use super::base::{Provider, TokenSet};
use super::oidc::ProviderOIDC;
use crate::config::Config;
use crate::error::Error;

pub struct ProviderKeycloak {
	oidc: ProviderOIDC,
}

impl ProviderKeycloak {
	///
	/// Create a new keycloak provider
	///
	pub fn new(config: &Config) -> Result<Self, Error> {
		let oidc = ProviderOIDC::new(config)?;
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
		Ok(Some(userinfo.unwrap().claims))
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
