use super::base::Provider;

#[derive(Debug)]
pub struct ProviderOIDC {}

impl ProviderOIDC {
	pub fn new() -> Self {
		Self {}
	}
}

#[async_trait::async_trait]
impl Provider for ProviderOIDC {
	async fn grant_authorization_code(&self) -> u32 {
		32
	}
}
