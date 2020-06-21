mod config;
mod error;
mod server;
mod util;

use actix_web::{App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	HttpServer::new(|| {
		let handler = server::handler::Handler::new();
		App::new().configure(|cfg| handler.config(cfg))
	})
	.bind("127.0.0.1:8088")?
	.run()
	.await
}
