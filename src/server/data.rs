use crate::config::Config;
use crate::error::Error;
use crate::provider::{create_provider, Provider};
use crate::util::crypto::{Crypto, RandomPtr};

#[allow(dead_code)]
pub struct Data {
	random: RandomPtr,
	pub config: Config,
	pub crypto: Crypto,
	pub provider: Box<dyn Provider>,
}

impl Data {
	pub fn new(config: Config, random: RandomPtr) -> Result<Self, Error> {
		let crypto = Crypto::new(&config.secret.as_ref().unwrap(), random.clone());
		let provider = create_provider(&config)?;
		Ok(Self {
			random: random,
			config: config,
			crypto: crypto,
			provider: provider,
		})
	}
}
