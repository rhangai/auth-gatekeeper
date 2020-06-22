#[async_trait::async_trait]
pub trait Provider: std::fmt::Debug {
	async fn grant_authorization_code(&self) -> u32;
}
