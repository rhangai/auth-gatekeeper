use crate::error::Error;

///
/// The token set
///
pub struct TokenSet {
	pub access_token: String,
	pub refresh_token: String,
}

#[async_trait::async_trait]
pub trait Provider {
	///
	/// Get the authorization url
	///
	fn get_authorization_url(&self, state: String) -> String;
	///
	/// Get the userinfo according to the access_token
	///
	async fn userinfo(&self, access_token: &str) -> Result<Option<serde_json::Value>, Error>;
	///
	/// Perform a grant_type: authorization_code request
	///
	async fn grant_authorization_code(&self, code: &str) -> Result<Option<TokenSet>, Error>;
	///
	/// Perform a new grant_type: refresh_token request
	///
	async fn grant_refresh_token(&self, refresh_token: &str) -> Result<Option<TokenSet>, Error>;
}
