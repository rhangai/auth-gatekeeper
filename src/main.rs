mod util;

#[tokio::main]
async fn main() {
	let c = util::crypto::Crypto::new("Senha");
	let encrypted = c.encrypt("oi").await.unwrap();
	println!("Encrypted value {}", encrypted);
}
