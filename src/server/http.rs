use super::data::Data;
use crate::error::Error;
use crate::provider::TokenSet;
use actix_http::ResponseBuilder;
use actix_web::{cookie, web, HttpMessage, HttpRequest};

pub struct HttpRequestRefreshInfo {
	pub token_set: Option<TokenSet>,
	pub userinfo: serde_json::Value,
}

pub struct Http {}

impl Http {
	///
	/// Request an userinfo and performs a refresh_token if needed
	///
	pub async fn request_refresh_info(
		req: &HttpRequest,
		data: &web::Data<Data>,
	) -> Result<Option<HttpRequestRefreshInfo>, Error> {
		// Check for the token_set
		let token_set_result = Self::request_get_token_set(&req, &data).unwrap_or(None);
		if token_set_result.is_none() {
			return Ok(None);
		}
		let token_set = token_set_result.unwrap();

		// Check for the userinfo, if everything is ok, return it
		let userinfo = data.provider.userinfo(&token_set.access_token).await?;
		if userinfo.is_some() {
			return Ok(Some(HttpRequestRefreshInfo {
				token_set: None,
				userinfo: userinfo.unwrap(),
			}));
		}

		// Try to get another refresh_token
		let new_token_set_result = data
			.provider
			.grant_refresh_token(&token_set.refresh_token)
			.await?;
		if new_token_set_result.is_none() {
			return Ok(None);
		}
		let new_token_set = new_token_set_result.unwrap();

		// Get the userinfo with the new token
		let userinfo = data.provider.userinfo(&new_token_set.access_token).await?;
		if userinfo.is_none() {
			return Ok(None);
		}

		// Return the token
		return Ok(Some(HttpRequestRefreshInfo {
			token_set: Some(new_token_set),
			userinfo: userinfo.unwrap(),
		}));
	}
	///
	/// Get the token set from this request
	///
	pub fn request_get_token_set(
		req: &HttpRequest,
		data: &web::Data<Data>,
	) -> Result<Option<TokenSet>, Error> {
		let cookies_result = req.cookies();
		if cookies_result.is_err() {
			return Ok(None);
		}
		let cookies = cookies_result.unwrap();

		let mut access_token: Option<String> = None;
		let mut refresh_token: Option<String> = None;
		for cookie in cookies.iter() {
			if cookie.name() == data.config.cookie_access_token_name {
				access_token = Some(data.crypto.decrypt(cookie.value())?);
			} else if cookie.name() == data.config.cookie_refresh_token_name {
				refresh_token = Some(data.crypto.decrypt(cookie.value())?);
			}
			if access_token.is_some() && refresh_token.is_some() {
				return Ok(Some(TokenSet {
					access_token: access_token.unwrap(),
					refresh_token: refresh_token.unwrap(),
					expires_in: None,
				}));
			}
		}
		Ok(None)
	}
	///
	/// Add the cookies from the token set to the response
	///
	pub fn response_set_userinfo(
		builder: &mut ResponseBuilder,
		data: &web::Data<Data>,
		userinfo: &serde_json::Value,
	) -> Result<(), Error> {
		if data.config.jwt_secret.is_some() {
			let auth_userinfo = jsonwebtoken::encode(
				&jsonwebtoken::Header::default(),
				&userinfo,
				&jsonwebtoken::EncodingKey::from_secret(
					data.config.jwt_secret.as_ref().unwrap().as_ref(),
				),
			);
			if auth_userinfo.is_err() {
				return Err(Error::ConfigError);
			}
			builder.header("x-auth-userinfo", auth_userinfo.unwrap());
		} else {
			builder.header("x-auth-userinfo", userinfo.to_string());
		}
		Ok(())
	}
	///
	/// Add the cookies from the token set to the response
	///
	pub fn response_add_cookies(
		builder: &mut ResponseBuilder,
		data: &web::Data<Data>,
		token_set: &TokenSet,
	) -> Result<(), Error> {
		let cookie_access_token = Self::create_cookie(
			data,
			&data.config.cookie_access_token_name,
			&token_set.access_token,
			token_set.expires_in,
		)?;
		let cookie_refresh_token = Self::create_cookie(
			data,
			&data.config.cookie_refresh_token_name,
			&token_set.refresh_token,
			None,
		)?;
		builder.cookie(cookie_access_token);
		builder.cookie(cookie_refresh_token);
		Ok(())
	}
	///
	/// Add the cookies from the token set to the response
	///
	pub fn response_add_x_headers(
		builder: &mut ResponseBuilder,
		data: &web::Data<Data>,
		token_set: &TokenSet,
	) -> Result<(), Error> {
		let cookie_access_token = Self::create_cookie(
			data,
			&data.config.cookie_access_token_name,
			&token_set.access_token,
			token_set.expires_in,
		)?;
		let cookie_refresh_token = Self::create_cookie(
			data,
			&data.config.cookie_refresh_token_name,
			&token_set.refresh_token,
			None,
		)?;
		builder.header("x-auth-set-cookie-1", cookie_access_token.to_string());
		builder.header("x-auth-set-cookie-2", cookie_refresh_token.to_string());
		Ok(())
	}

	fn create_cookie<'a>(
		data: &web::Data<Data>,
		name: &str,
		value: &str,
		_expires_in: Option<i64>,
	) -> Result<cookie::Cookie<'a>, Error> {
		let cookie_value = data.crypto.encrypt(value)?;
		let builder = cookie::Cookie::build(name.to_owned(), cookie_value).path("/");
		Ok(builder.finish())
	}
}
