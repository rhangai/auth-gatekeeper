use crate::config::Config;
use crate::error::Error;
use actix_web::{HttpRequest, HttpResponse};

pub struct ResponseClear {}

impl ResponseClear {
	pub fn new(config: &Config) -> Self {
		Self {}
		// Self { config: config }
	}
}

impl actix_web::Responder for ResponseClear {
	type Error = crate::error::Error;
	type Future = futures::future::Ready<Result<HttpResponse, Error>>;

	fn respond_to(self, _req: &HttpRequest) -> Self::Future {
		let response = HttpResponse::Unauthorized().finish();
		futures::future::ready(Ok(response))
	}
	// type Future = futures::future::Ready;
}
