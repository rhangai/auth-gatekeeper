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

enum SessionStatus {
	Invalid,
	New(Option<Userinfo>),
	Logged(Option<Userinfo>),
	Logout,
}

bitflags! {
	pub struct SessionFlags: u8 {
		const X_AUTH_HEADERS = 0x01;
		const COOKIES   = 0x02;
	}
}

pub struct Session {
	data: web::Data<Data>,
	status: SessionStatus,
	has_session: bool,
	token_set: Option<SessionTokenSet>,
	id_token: Option<serde_json::Value>,
}

impl Session {
	pub fn new(data: web::Data<Data>, token_set: TokenSet) -> Self {
		Self {
			data: data,
			status: SessionStatus::New(None),
			token_set: Some(SessionTokenSet {
				access_token: Some(token_set.access_token),
				refresh_token: Some(token_set.refresh_token),
			}),
			has_session: false,
			id_token: token_set.id_token,
		}
	}

	pub fn logout(data: web::Data<Data>) -> Self {
		Self {
			data: data,
			status: SessionStatus::Logout,
			token_set: None,
			has_session: true,
			id_token: None,
		}
	}

	pub fn from_request(data: web::Data<Data>, req: &HttpRequest) -> Self {
		let token_set = Self::request_get_token_set(&data, &req);
		let has_session = token_set.is_some();
		Self {
			data: data,
			status: SessionStatus::Invalid,
			token_set: token_set,
			has_session: has_session,
			id_token: None,
		}
	}

	/// Get the token set from the request
	fn request_get_token_set(data: &web::Data<Data>, req: &HttpRequest) -> Option<SessionTokenSet> {
		let cookies_result = req.cookies();
		if cookies_result.is_err() {
			return None;
		}
		let cookies = cookies_result.unwrap();

		let mut access_token: Option<&str> = None;
		let mut refresh_token: Option<&str> = None;
		for cookie in cookies.iter() {
			if cookie.name() == data.settings.cookie.access_token_name {
				access_token = Some(cookie.value());
			} else if cookie.name() == data.settings.cookie.refresh_token_name {
				refresh_token = Some(cookie.value());
			}
			if access_token.is_some() && refresh_token.is_some() {
				break;
			}
		}
		if access_token.is_none() && refresh_token.is_none() {
			return None;
		}
		let access_token = if let Some(access_token) = access_token {
			data.crypto.decrypt(access_token).ok()
		} else {
			None
		};
		let refresh_token = if let Some(refresh_token) = refresh_token {
			data.crypto.decrypt(refresh_token).ok()
		} else {
			None
		};
		return Some(SessionTokenSet {
			access_token: access_token,
			refresh_token: refresh_token,
		});
	}

	///
	/// Check if any api calls are necessary
	///
	async fn api_id_token(&self, cookies: &mut Vec<cookie::Cookie<'static>>) -> Result<(), Error> {
		if let Some(ref id_token) = self.id_token {
			let id_token = self.data.jwt.encode_value(id_token)?;
			self.data.api.on_id_token(cookies, &id_token).await?;
		}
		Ok(())
	}

	///
	/// Check if any api calls are necessary
	///
	async fn api_logout(&self, cookies: &mut Vec<cookie::Cookie<'static>>) -> Result<(), Error> {
		self.data.api.on_logout(cookies).await?;
		Ok(())
	}

	///
	/// Get the userinfo
	///
	pub fn get_userinfo<'a>(&'a self) -> Option<&'a Userinfo> {
		match self.status {
			SessionStatus::Invalid => None,
			SessionStatus::Logout => None,
			SessionStatus::New(ref userinfo) => userinfo.as_ref(),
			SessionStatus::Logged(ref userinfo) => userinfo.as_ref(),
		}
	}
	///
	/// Validate the information and try to refresh the session
	///
	pub async fn validate(&mut self, refresh: bool) -> Result<(), Error> {
		// Invalidates the session
		self.status = SessionStatus::Invalid;

		// If there is no token, then it is already invalid
		if self.token_set.is_none() {
			return Ok(());
		}

		// If there is a token set already, try to load the userinfo
		let token_set = self.token_set.clone().unwrap();
		if let Some(access_token) = token_set.access_token {
			let userinfo = self.data.provider.userinfo(&access_token).await?;
			if userinfo.is_some() {
				self.status = SessionStatus::Logged(userinfo);
				return Ok(());
			}
		}

		// Check if need to refresh
		if refresh {
			if let Some(refresh_token) = token_set.refresh_token {
				let new_token_set_result = self
					.data
					.provider
					.grant_refresh_token(&refresh_token)
					.await?;
				if let Some(new_token_set) = new_token_set_result {
					let userinfo = self
						.data
						.provider
						.userinfo(&new_token_set.access_token)
						.await?;
					if userinfo.is_some() {
						self.token_set = Some(SessionTokenSet {
							access_token: Some(new_token_set.access_token),
							refresh_token: Some(new_token_set.refresh_token),
						});
						self.id_token = new_token_set.id_token;
						self.status = SessionStatus::New(userinfo);
					}
				}
			}
		}
		Ok(())
	}
	///
	/// Build the response using the flags
	///
	/// If SessionFlags::COOKIES is requested, allow the set-cookie headers
	/// If SessionFlags::X_AUTH_HEADERS is requested, then set the x-auth headers
	///
	pub async fn response(
		&self,
		builder: &mut ResponseBuilder,
		flags: SessionFlags,
	) -> Result<(), Error> {
		let mut cookies: Vec<cookie::Cookie<'static>> = Vec::new();
		match self.status {
			SessionStatus::Invalid => {
				if self.has_session {
					self.response_save_session(&mut cookies, None)?;
					self.api_logout(&mut cookies).await?;
				}
				builder.status(StatusCode::UNAUTHORIZED);
			}
			SessionStatus::Logout => {
				self.response_save_session(&mut cookies, None)?;
				self.api_logout(&mut cookies).await?;
			}
			SessionStatus::New(ref userinfo) => {
				self.response_save_session(&mut cookies, self.token_set.clone())?;
				self.response_set_userinfo(builder, &userinfo, flags)?;
				self.api_id_token(&mut cookies).await?;
			}
			SessionStatus::Logged(ref userinfo) => {
				self.response_set_userinfo(builder, &userinfo, flags)?;
			}
		}
		if flags.contains(SessionFlags::X_AUTH_HEADERS) {
			let mut i = 1;
			for cookie in &cookies {
				builder.header(&format!("x-auth-set-cookie-{}", i), cookie.to_string());
				i += 1;
			}
		}
		if flags.contains(SessionFlags::COOKIES) {
			for cookie in cookies {
				builder.cookie(cookie);
			}
		}
		Ok(())
	}
	///
	/// Save the userinfo
	///
	fn response_set_userinfo(
		&self,
		builder: &mut ResponseBuilder,
		userinfo: &Option<Userinfo>,
		flags: SessionFlags,
	) -> Result<(), Error> {
		if let Some(ref userinfo) = userinfo {
			if flags.contains(SessionFlags::X_AUTH_HEADERS) {
				let userinfo_encoded = self.data.jwt.encode_str(&userinfo.data)?;
				builder.header("x-auth-userinfo", userinfo_encoded);
				if let Some(ref data) = self.data.settings.data {
					builder.header("x-auth-data", data.clone());
				};
			}
		}
		Ok(())
	}
	///
	/// Save the session
	///
	/// When the session doesn't have a session token
	///
	fn response_save_session(
		&self,
		cookies: &mut Vec<cookie::Cookie<'static>>,
		token_set: Option<SessionTokenSet>,
	) -> Result<(), Error> {
		// If the token is not set, then clear the session
		if token_set.is_none() {
			let cookie_access_token_name = self.data.settings.cookie.access_token_name.clone();
			let cookie_access_token = self.create_cookie(cookie_access_token_name, None)?;
			let cookie_refresh_token_name = self.data.settings.cookie.refresh_token_name.clone();
			let cookie_refresh_token = self.create_cookie(cookie_refresh_token_name, None)?;
			cookies.push(cookie_access_token);
			cookies.push(cookie_refresh_token);
			return Ok(());
		}

		let token_set = token_set.unwrap();
		let cookie_access_token_name = self.data.settings.cookie.access_token_name.clone();
		let cookie_access_token =
			self.create_cookie(cookie_access_token_name, token_set.access_token)?;
		let cookie_refresh_token_name = self.data.settings.cookie.refresh_token_name.clone();
		let cookie_refresh_token =
			self.create_cookie(cookie_refresh_token_name, token_set.refresh_token)?;
		cookies.push(cookie_access_token);
		cookies.push(cookie_refresh_token);
		Ok(())
	}

	///
	/// Create a cookie to be used. If None is passed, the cookie is marked as deleted
	///
	fn create_cookie(
		&self,
		name: String,
		value: Option<String>,
	) -> Result<cookie::Cookie<'static>, Error> {
		let cookie_value = if let Some(ref v) = value {
			self.data.crypto.encrypt(v)?
		} else {
			String::from("")
		};
		let mut builder = cookie::Cookie::build(name, cookie_value)
			.path("/")
			.http_only(true);
		if value.is_none() {
			builder = builder.expires(time::at_utc(time::Timespec::new(1, 0)));
		}
		Ok(builder.finish())
	}
}
