use super::base::{Provider, ProviderGrantAuthorizationCodeForm, TokenSet};

pub struct ProviderOIDC {}

impl ProviderOIDC {
	pub fn new() -> Self {
		ProviderOIDC {}
	}
}

#[async_trait::async_trait]
impl Provider for ProviderOIDC {
	async fn grant_authorization_code(form: ProviderGrantAuthorizationCodeForm) -> TokenSet {
		TokenSet::new()
	}
}
