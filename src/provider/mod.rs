mod base;
mod keycloak;
mod oidc;
use crate::config::Config;
use crate::error::Error;
pub use base::*;
pub use keycloak::ProviderKeycloak;
pub use oidc::ProviderOIDC;

pub fn create_provider(config: &Config) -> Result<Box<dyn Provider>, Error> {
	if config.provider == "keycloak" {
		Ok(Box::new(ProviderKeycloak::new(&config)?))
	} else if config.provider == "oidc" {
		Ok(Box::new(ProviderOIDC::new(&config)?))
	} else {
		Err(Error::ConfigError)
	}
}
