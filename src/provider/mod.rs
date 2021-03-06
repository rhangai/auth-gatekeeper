mod base;
mod oidc;
use crate::error::Error;
use crate::settings::Settings;
pub use base::*;
pub use oidc::{ProviderOIDC, ProviderOIDCOptions};

pub enum ProviderBox {
	OIDC(ProviderOIDC),
}

impl ProviderBox {
	///
	/// Get the authorization url
	///
	pub fn get_authorization_url(&self, state: String) -> String {
		match self {
			ProviderBox::OIDC(provider) => provider.get_authorization_url(state),
		}
	}
	///
	/// Get the logout url
	///
	pub fn get_logout_url(&self) -> String {
		match self {
			ProviderBox::OIDC(provider) => provider.get_logout_url(),
		}
	}
	///
	/// Get the userinfo according to the access_token
	///
	pub async fn userinfo(&self, access_token: &str) -> Result<Option<Userinfo>, Error> {
		match self {
			ProviderBox::OIDC(provider) => provider.userinfo(access_token).await,
		}
	}
	///
	/// Perform a grant_type: authorization_code request
	///
	pub async fn grant_authorization_code(&self, code: &str) -> Result<Option<TokenSet>, Error> {
		match self {
			ProviderBox::OIDC(provider) => provider.grant_authorization_code(code).await,
		}
	}
	///
	/// Perform a grant_type: password request
	///
	pub async fn grant_password(
		&self,
		username: &str,
		password: &str,
	) -> Result<Option<TokenSet>, Error> {
		match self {
			ProviderBox::OIDC(provider) => provider.grant_password(username, password).await,
		}
	}
	///
	/// Perform a new grant_type: refresh_token request
	///
	pub async fn grant_refresh_token(
		&self,
		refresh_token: &str,
	) -> Result<Option<TokenSet>, Error> {
		match self {
			ProviderBox::OIDC(provider) => provider.grant_refresh_token(refresh_token).await,
		}
	}
}

pub fn create_provider(settings: &Settings) -> Result<ProviderBox, Error> {
	if settings.provider.provider == "keycloak" || settings.provider.provider == "fusionauth" {
		let provider = ProviderOIDC::new(
			&settings,
			ProviderOIDCOptions {
				userinfo_from_access_token: true,
			},
		)?;
		Ok(ProviderBox::OIDC(provider))
	} else if settings.provider.provider == "oidc" {
		let provider = ProviderOIDC::new(
			&settings,
			ProviderOIDCOptions {
				userinfo_from_access_token: false,
			},
		)?;
		Ok(ProviderBox::OIDC(provider))
	} else {
		Err(Error::SettingsError("Invalid provider"))
	}
}
