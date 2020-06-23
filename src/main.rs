#[macro_use]
extern crate envconfig_derive;

mod config;
mod error;
mod provider;
mod server;
mod util;

use actix_web::{App, HttpServer};
use reqwest::Url;

///
/// Show help
///
fn show_help() {
	let help = r###"
Environment Variables:
- AUTH_GATEKEEPER_LISTEN
    Urls to listen to. Ex: http://localhost:8088,unix:/path/to/sock.sock
- AUTH_GATEKEEPER_SECRET
    Secret to encrypt the cookies
- AUTH_GATEKEEPER_JWT_SECRET
    Secret to encode the headers in x-auth-userinfo and x-auth-id-token
- AUTH_GATEKEEPER_PROVIDER
    Provider for the gatekeeper. "oidc" or "keycloak"
- AUTH_GATEKEEPER_PROVIDER_CLIENT_ID
    The client id for the provider
- AUTH_GATEKEEPER_PROVIDER_CLIENT_SECRET
    The client id for the secret
- AUTH_GATEKEEPER_PROVIDER_AUTH_URL
    The
- AUTH_GATEKEEPER_PROVIDER_TOKEN_URL
- AUTH_GATEKEEPER_PROVIDER_USERINFO_URL
- AUTH_GATEKEEPER_PROVIDER_CALLBACK_URL
"###;
	print!("Usage auth-gatekeeper\n{}", help);
}

///
/// Entrypoint
///
/// Initializes the server and print the configuration help if needed
///
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	env_logger::init();

	let random = util::crypto::Crypto::create_random();
	let config = config::Config::parse(random.clone());
	if config.is_err() {
		show_help();
		std::process::exit(1);
	}

	let config = config.unwrap();
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
			log::info!("Listening on http://{}", addr);
			server = server.bind(addr)?;
		} else if url.scheme() == "unix" {
			log::info!("Listening on unix:{}", url.path());
			server = server.bind_uds(url.path())?;
		} else {
			panic!("Invalid listen url: {}", listen);
		}
	}
	server.run().await
}
