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

bitflags! {
	pub struct SessionFlags: u8 {
		const X_HEADERS = 0x01;
		const COOKIES   = 0x02;
	}
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

	///
	/// Validate the information and try to refresh the session
	///
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
	/// Set the response
	///
	pub fn response(
		&self,
		builder: &mut ResponseBuilder,
		flags: SessionFlags,
	) -> Result<(), Error> {
		if self.userinfo.is_none() {
			if self.has_session {
				self.response_save_session(builder, None, flags)?;
			}
			builder.status(StatusCode::UNAUTHORIZED);
			return Ok(());
		}
		if self.token_set_renewed {
			self.response_save_session(builder, self.token_set.clone(), flags)?;
		}
		Ok(())
	}
	///
	/// Save the
	///
	fn response_save_session(
		&self,
		builder: &mut ResponseBuilder,
		token_set: Option<SessionTokenSet>,
		flags: SessionFlags,
	) -> Result<(), Error> {
		// If the token
		if token_set.is_none() {
			let cookie_access_token_name = self.data.settings.cookie.access_token_name.clone();
			let cookie_access_token = self.create_cookie(cookie_access_token_name, None)?;
			let cookie_refresh_token_name = self.data.settings.cookie.refresh_token_name.clone();
			let cookie_refresh_token = self.create_cookie(cookie_refresh_token_name, None)?;
			if flags.contains(SessionFlags::X_HEADERS) {
				builder.header("x-auth-set-cookie-1", cookie_access_token.to_string());
				builder.header("x-auth-set-cookie-2", cookie_refresh_token.to_string());
			}
			if flags.contains(SessionFlags::COOKIES) {
				builder.cookie(cookie_access_token);
				builder.cookie(cookie_refresh_token);
			}
			return Ok(());
		}

		let token_set = token_set.unwrap();
		let cookie_access_token_name = self.data.settings.cookie.access_token_name.clone();
		let cookie_access_token =
			self.create_cookie(cookie_access_token_name, token_set.access_token)?;

		// Set the cookie
		let cookie_refresh_token_name = self.data.settings.cookie.refresh_token_name.clone();
		let cookie_refresh_token =
			self.create_cookie(cookie_refresh_token_name, token_set.refresh_token)?;
		if flags.contains(SessionFlags::X_HEADERS) {
			builder.header("x-auth-set-cookie-1", cookie_access_token.to_string());
			builder.header("x-auth-set-cookie-2", cookie_refresh_token.to_string());
		}
		if flags.contains(SessionFlags::COOKIES) {
			builder.cookie(cookie_access_token);
			builder.cookie(cookie_refresh_token);
		}
		Ok(())
	}

	fn create_cookie(&self, name: String, value: Option<String>) -> Result<cookie::Cookie, Error> {
		let cookie_value = if let Some(ref v) = value {
			self.data.crypto.encrypt(v)?
		} else {
			String::from("")
		};
		let mut builder = cookie::Cookie::build(name, cookie_value).path("/");
		if value.is_none() {
			// builder.expires(std::time::SystemTime::UNIX_EPOCH);
			builder = builder.expires(time::empty_tm());
		}
		Ok(builder.finish())
	}
}
