#[path = "./options.rs"]
mod options;
#[path = "../util/mod.rs"]
mod util;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use util::crypto::{Crypto, RandomPtr};

#[derive(Clone)]
pub struct State {
	random: RandomPtr,
	config: options::Config,
	crypto: Crypto,
}

impl State {
	pub fn new(config: options::Config, random: RandomPtr) -> State {
		let crypto = Crypto::new(&config.cookie_secret.clone(), random.clone());
		State {
			random: random,
			config: config,
			crypto: crypto,
		}
	}
}
