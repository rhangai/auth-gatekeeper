use crate::config::Config;
use crate::error::Error;
use crate::provider::base::Provider;
use crate::provider::oidc::ProviderOIDC;
use crate::util::crypto::{Crypto, RandomPtr};

pub struct Data {
	random: RandomPtr,
	pub config: Config,
	pub crypto: Crypto,
	pub provider: Box<dyn Provider>,
}

impl Data {
	pub fn new(config: Config, random: RandomPtr) -> Result<Self, Error> {
		let crypto = Crypto::new("test", random.clone());
		let provider = ProviderOIDC::new()?;
		Ok(Self {
			random: random,
			config: config,
			crypto: crypto,
			provider: Box::new(provider),
		})
	}
}
