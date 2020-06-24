mod base;
mod keycloak;
mod oidc;
use crate::error::Error;
use crate::settings::Settings;
pub use base::*;
pub use keycloak::ProviderKeycloak;
pub use oidc::ProviderOIDC;

pub fn create_provider(settings: &Settings) -> Result<Box<dyn Provider>, Error> {
	if settings.provider.provider == "keycloak" {
		Ok(Box::new(ProviderKeycloak::new(&settings)?))
	} else if settings.provider.provider == "oidc" {
		Ok(Box::new(ProviderOIDC::new(&settings)?))
	} else {
		Err(Error::SettingsError("Invalid provider"))
	}
}
