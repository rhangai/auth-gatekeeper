use crate::error::Error;
use crate::provider::{create_provider, Provider};
use crate::settings::Settings;
use crate::util::crypto::{Crypto, RandomPtr};

#[allow(dead_code)]
pub struct Data {
	random: RandomPtr,
	pub settings: Settings,
	pub crypto: Crypto,
	pub provider: Box<dyn Provider>,
}

impl Data {
	pub fn new(settings: Settings, random: RandomPtr) -> Result<Self, Error> {
		let crypto = Crypto::new(&settings.secret, random.clone());
		let provider = create_provider(&settings)?;
		Ok(Self {
			random: random,
			settings: settings,
			crypto: crypto,
			provider: provider,
		})
	}
}
