///
/// The token set
///
#[derive(Debug)]
pub struct TokenSet {
	pub access_token: String,
	pub refresh_token: String,
	pub expires_in: Option<i64>,
	pub id_token: Option<serde_json::Value>,
}

///
/// Userinfo
///
#[derive(Debug)]
pub struct Userinfo {
	pub data: serde_json::Value,
	pub expires_at: Option<std::time::SystemTime>,
}

pub trait Provider {
	///
	/// Get the authorization url
	///
	fn get_authorization_url(&self, state: String) -> String;
	///
	/// Get the logout url
	///
	fn get_logout_url(&self) -> String;
}
