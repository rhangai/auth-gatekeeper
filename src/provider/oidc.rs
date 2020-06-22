use super::base::{Provider, TokenSet};
use crate::error::Error;
use reqwest::Url;

#[derive(Debug)]
pub struct ProviderOIDC {
	auth_url: Url,
}

impl ProviderOIDC {
	pub fn new() -> Result<Self, Error> {
		let auth_url =
			Url::parse("https://docs.rs/reqwest/0.10.6/reqwest/struct.Url.html#method.into_string")
				.or_else(|_| Err(Error::CryptoError))?;

		Ok(Self { auth_url: auth_url })
	}

	async fn grant<T: serde::Serialize + ?Sized>(&self, form: &T) -> Result<TokenSet, Error> {
		let client = reqwest::Client::new();
		let res = client
			.post("http://httpbin.org/post")
			.form(form)
			.send()
			.await;
		println!("Request {:?}", res);
		let body = res.unwrap().json::<serde_json::Value>().await;
		println!("Body {:?}", body);
		Ok(TokenSet {
			access_token: String::from("oi"),
		})
	}
}

#[async_trait::async_trait]
impl Provider for ProviderOIDC {
	fn get_authorization_url(&self, state: String) -> String {
		let mut url = self.auth_url.clone();
		let mut query_pairs = url.query_pairs_mut();
		query_pairs
			.append_pair("response_type", "code")
			.append_pair("scope", "openid email profile")
			.append_pair("client_id", "");
		if !state.is_empty() {
			query_pairs.append_pair("state", &state);
		}
		drop(query_pairs);
		url.into_string()
	}

	async fn grant_authorization_code(&self) -> Result<TokenSet, Error> {
		let params = [
			("grant_type", "authorization_code"),
			("client_id", "teste"),
			("client_secret", "teste"),
			("redirect_uri", "teste"),
		];
		self.grant(&params).await
	}

	async fn grant_refresh_token(&self) -> Result<TokenSet, Error> {
		let params = [
			("grant_type", "refresh_token"),
			("client_id", "teste"),
			("client_secret", "teste"),
		];
		self.grant(&params).await
	}
}
