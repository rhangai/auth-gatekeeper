use crate::api::Api;
use crate::error::Error;
use crate::provider::{create_provider, Provider};
use crate::settings::Settings;
use crate::util::crypto::{Crypto, RandomPtr};
use crate::util::jwt::JWT;

#[allow(dead_code)]
pub struct Data {
	random: RandomPtr,
	pub settings: Settings,
	pub crypto: Crypto,
	pub jwt: JWT,
	pub api: Api,
	pub provider: Box<dyn Provider>,
}

impl Data {
	pub fn new(settings: Settings, random: RandomPtr) -> Result<Self, Error> {
		let crypto = Crypto::new(&settings.secret, random.clone());
		let jwt = JWT::new(settings.jwt_secret.clone())?;
		let api = Api::new(&settings)?;
		let provider = create_provider(&settings)?;
		Ok(Self {
			random: random,
			settings: settings,
			crypto: crypto,
			jwt: jwt,
			api: api,
			provider: provider,
		})
	}
}
