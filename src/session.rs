use super::error::Error;
use super::provider::{TokenSet, Userinfo};
use super::server::data::Data;
use actix_http::{http::StatusCode, ResponseBuilder};
use actix_web::{cookie, web, HttpMessage, HttpRequest};

#[derive(Clone)]
struct SessionTokenSet {
	access_token: Option<String>,
	refresh_token: Option<String>,
}

pub struct Session {
	data: web::Data<Data>,
	token_set: Option<SessionTokenSet>,
	token_set_renewed: bool,
	has_session: bool,
	userinfo: Option<Userinfo>,
}

impl Session {
	pub fn new(data: web::Data<Data>, token_set: TokenSet) -> Self {
		Self {
			data: data,
			token_set: Some(SessionTokenSet {
				access_token: Some(token_set.access_token),
				refresh_token: Some(token_set.refresh_token),
			}),
			token_set_renewed: false,
			has_session: false,
			userinfo: None,
		}
	}

	pub fn from_request(data: web::Data<Data>, req: &HttpRequest) -> Self {
		let token_set = Self::request_get_token_set(&data, &req);
		let has_session = token_set.is_some();
		Self {
			data: data,
			token_set: token_set,
			token_set_renewed: false,
			has_session: has_session,
			userinfo: None,
		}
	}

	/// Get the token set from the request
	fn request_get_token_set(data: &web::Data<Data>, req: &HttpRequest) -> Option<SessionTokenSet> {
		let cookies_result = req.cookies();
		if cookies_result.is_err() {
			return None;
		}
		let cookies = cookies_result.unwrap();

		let mut access_token: Option<String> = None;
		let mut refresh_token: Option<String> = None;
		for cookie in cookies.iter() {
			if cookie.name() == data.settings.cookie.access_token_name {
				access_token = data.crypto.decrypt(cookie.value()).ok();
			} else if cookie.name() == data.settings.cookie.refresh_token_name {
				refresh_token = data.crypto.decrypt(cookie.value()).ok();
			}
			if access_token.is_some() && refresh_token.is_some() {
				break;
			}
		}
		if access_token.is_none() && refresh_token.is_none() {
			return None;
		}
		return Some(SessionTokenSet {
			access_token: access_token,
			refresh_token: refresh_token,
		});
	}

	/// Try to load the userinfo
	async fn load_userinfo(&mut self, access_token: &str) -> Result<bool, Error> {
		let userinfo = self.data.provider.userinfo(&access_token).await?;
		if userinfo.is_none() {
			return Ok(false);
		}
		self.userinfo = userinfo;
		Ok(true)
	}

	/// Try to load the userinfo
	pub async fn validate(&mut self) -> Result<(), Error> {
		if self.token_set.is_none() {
			return Ok(());
		}

		// If there is a token set already, try to load the userinfo
		let token_set = self.token_set.clone().unwrap();
		if let Some(access_token) = token_set.access_token {
			let has_userinfo = self.load_userinfo(&access_token).await?;
			if has_userinfo {
				return Ok(());
			}
		}

		// Try to get a new refresh token
		if let Some(refresh_token) = token_set.refresh_token {
			let new_token_set_result = self
				.data
				.provider
				.grant_refresh_token(&refresh_token)
				.await?;
			if let Some(new_token_set) = new_token_set_result {
				let has_userinfo = self.load_userinfo(&new_token_set.access_token).await?;
				if has_userinfo {
					self.token_set = Some(SessionTokenSet {
						access_token: Some(new_token_set.access_token),
						refresh_token: Some(new_token_set.refresh_token),
					});
					self.token_set_renewed = true;
				}
			}
		}

		Ok(())
	}

	///
	///
	pub fn response(&self, builder: &mut ResponseBuilder, flags: u32) {
		if self.userinfo.is_none() {
			builder.status(StatusCode::UNAUTHORIZED);
			return;
		}
	}

	// pub fn set_cookies(&self, builder: &mut ResponseBuilder) {
	// 	if let Some(ref cookies) = self.cookies {
	// 		for cookie in cookies {
	// 			builder.cookie(cookie.clone());
	// 		}
	// 	}
	// }

	// ///
	// /// Set the X-Auth headers on the given object
	// ///
	// pub fn set_x_auth_headers(&self, builder: &mut ResponseBuilder) {
	// 	if let Some(ref userinfo) = self.userinfo {
	// 		// builder.header("x-auth-userinfo", userinfo.data);
	// 	}
	// 	if let Some(ref cookies) = self.cookies {
	// 		let mut i = 1;
	// 		for cookie in cookies {
	// 			builder.header(&format!("x-auth-set-cookie-{}", i), cookie.to_string());
	// 			i += 1;
	// 		}
	// 	}
	// }
}
