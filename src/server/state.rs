use crate::config::Config;
use crate::provider::base::Provider;
use crate::provider::oidc::ProviderOIDC;
use crate::util::crypto::{Crypto, RandomPtr};

pub struct State {
	random: RandomPtr,
	pub config: Config,
	pub crypto: Crypto,
	pub provider: Box<dyn Provider>,
}

impl State {
	pub fn new(config: Config, random: RandomPtr) -> Self {
		let crypto = Crypto::new("test", random.clone());
		Self {
			random: random,
			config: config,
			crypto: crypto,
			provider: Box::new(ProviderOIDC::new()),
		}
	}
}
