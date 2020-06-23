#[macro_use]
extern crate envconfig_derive;

mod config;
mod error;
mod provider;
mod server;
mod util;

use actix_web::{App, HttpServer};
use reqwest::Url;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	let random = util::crypto::Crypto::create_random();

	let config = config::Config::parse(random.clone());
	let listen = config.listen.clone();
	let mut server = HttpServer::new(move || {
		let handler = server::handler::Handler::new(random.clone(), config.clone()).unwrap();
		App::new().configure(|cfg| handler.config(cfg).unwrap())
	});

	let listen_list = listen.split_terminator(',');
	for listen in listen_list {
		let url = Url::parse(listen);
		if url.is_err() {
			panic!("Invalid listen url: {}", listen);
		}

		let url = url.unwrap();
		if url.scheme() == "http" {
			let addr = format!(
				"{}:{}",
				url.host().unwrap(),
				url.port_or_known_default().unwrap()
			);
			println!("Listening on http://{}", addr);
			server = server.bind(addr)?;
		} else if url.scheme() == "unix" {
			println!("Listening on unix:{}", url.path());
			server = server.bind_uds(url.path())?;
		} else {
			panic!("Invalid listen url: {}", listen);
		}
	}
	server.run().await
}
