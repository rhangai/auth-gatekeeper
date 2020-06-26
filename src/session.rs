use super::provider::Userinfo;
use actix_http::ResponseBuilder;
use actix_web::cookie;

pub struct Session {
	userinfo: Option<Userinfo>,
	cookies: Option<Vec<cookie::Cookie<'static>>>,
}

impl Session {
	pub fn new() -> Self {
		Self {
			userinfo: None,
			cookies: None,
		}
	}

	pub fn set_cookies(&self, builder: &mut ResponseBuilder) {
		if let Some(ref cookies) = self.cookies {
			for cookie in cookies {
				builder.cookie(cookie.clone());
			}
		}
	}

	///
	/// Set the X-Auth headers on the given object
	///
	pub fn set_x_auth_headers(&self, builder: &mut ResponseBuilder) {
		if let Some(ref userinfo) = self.userinfo {
			// builder.header("x-auth-userinfo", userinfo.data);
		}
		if let Some(ref cookies) = self.cookies {
			let mut i = 1;
			for cookie in cookies {
				builder.header(&format!("x-auth-set-cookie-{}", i), cookie.to_string());
				i += 1;
			}
		}
	}
}
