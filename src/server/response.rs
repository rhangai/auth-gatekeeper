use super::data::Data;
use crate::error::Error;
use crate::provider::TokenSet;
use actix_http::ResponseBuilder;
use actix_web::{cookie, web, HttpResponse};

pub struct Response {}

impl Response {
	///
	pub fn token_set_add_cookies(
		builder: &mut ResponseBuilder,
		data: &web::Data<Data>,
		token_set: &TokenSet,
	) {
		builder.cookie(cookie::Cookie::build("a", "b").finish());
	}
}
