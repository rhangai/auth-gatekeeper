#[macro_use]
extern crate bitflags;

mod api;
mod error;
mod provider;
mod server;
mod session;
mod settings;
mod util;

use actix_web::{http::Uri, App, HttpServer};

///
/// Entrypoint
///
/// Initializes the server and print the configuration help if needed
///
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init();
	let random = util::crypto::Crypto::create_random();
	let settings = settings::Settings::new(random.as_ref());
	let listen = settings.listen.clone();
	let mut server = HttpServer::new(move || {
		let handler = server::handler::Handler::new(random.clone(), settings.clone()).unwrap();
		App::new().configure(|cfg| handler.config(cfg).unwrap())
	});

	// Check the urls to listen to
	let listen_list = listen.split_terminator(',');
	for listen in listen_list {
		let url = listen.parse::<Uri>();
		if url.is_err() {
			panic!("Invalid listen url: {}", listen);
		}

		let url = url.unwrap();
		let scheme = url.scheme_str().unwrap_or("");
		if scheme == "http" {
			let addr = format!("{}:{}", url.host().unwrap(), url.port_u16().unwrap_or(80));
			log::info!("Listening on http://{}", addr);
			server = server.bind(addr)?;
		} else if scheme == "unix" {
			log::info!("Listening on unix:{}", url.path());
			server = server.bind_uds(url.path())?;
		} else {
			panic!("Invalid listen url: {}", listen);
		}
	}
	server.run().await
}
