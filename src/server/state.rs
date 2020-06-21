use crate::config;
use crate::util::crypto::{Crypto, RandomPtr};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

#[derive(Clone)]
pub struct State {
	random: RandomPtr,
	pub config: config::Config,
	pub crypto: Crypto,
}

impl State {
	pub fn new(config: config::Config, random: RandomPtr) -> State {
		let crypto = Crypto::new(&config.cookie_secret, random.clone());
		State {
			random: random,
			config: config,
			crypto: crypto,
		}
	}
}
