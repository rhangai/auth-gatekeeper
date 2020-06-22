use crate::error::Error;

///
/// The token set
///
pub struct TokenSet {
	pub access_token: String,
}

#[async_trait::async_trait]
pub trait Provider {
	///
	/// Get the authorization url
	///
	fn get_authorization_url(&self, state: String) -> String;
	///
	/// Perform a grant_type: authorization_code request
	///
	async fn grant_authorization_code(&self) -> Result<TokenSet, Error>;
	///
	/// Perform a new grant_type: refresh_token request
	///
	async fn grant_refresh_token(&self) -> Result<TokenSet, Error>;
}
