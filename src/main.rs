mod config;
mod error;
mod provider;
mod server;
mod util;

use actix_web::{App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	let config = config::Config::new();
	HttpServer::new(move || {
		let handler = server::handler::Handler::new(config.clone()).unwrap();
		App::new().configure(|cfg| handler.config(cfg).unwrap())
	})
	.bind("127.0.0.1:8088")?
	.run()
	.await
}
