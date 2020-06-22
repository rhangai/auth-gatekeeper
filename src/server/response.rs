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
	) -> Result<(), Error> {
		let cookie_access_token = Self::create_cookie(
			data,
			&data.config.cookie_access_token_name,
			&token_set.access_token,
		)?;
		let cookie_refresh_token = Self::create_cookie(
			data,
			&data.config.cookie_refresh_token_name,
			&token_set.access_token,
		)?;
		builder.cookie(cookie_access_token);
		builder.cookie(cookie_refresh_token);
		Ok(())
	}

	fn create_cookie<'a>(
		data: &web::Data<Data>,
		name: &str,
		value: &str,
	) -> Result<cookie::Cookie<'a>, Error> {
		let cookie_value = data.crypto.encrypt(value)?;
		let cookie = cookie::Cookie::build(name.to_owned(), cookie_value).finish();
		Ok(cookie)
	}
}
