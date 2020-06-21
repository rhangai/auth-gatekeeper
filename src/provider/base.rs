pub struct TokenSet {
	access_token: String,
}

pub struct ProviderGrantAuthorizationCodeForm {
	code: String,
}

#[async_trait::async_trait]
pub trait Provider {
	async fn grant_authorization_code(form: ProviderGrantAuthorizationCodeForm) -> TokenSet;
}

impl TokenSet {
	pub fn new() -> Self {
		TokenSet {
			access_token: String::from("oi"),
		}
	}
}
