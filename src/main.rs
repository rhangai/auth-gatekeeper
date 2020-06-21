mod util;

#[tokio::main]
async fn main() {
	let c = util::crypto::Crypto::new("Senha");
	let encrypted = c.encrypt("teste").unwrap();
	println!("Encrypted value {:?}", encrypted);
	let decrypted = c.decrypt(&encrypted).unwrap();
	println!("Decrypted value {:?}", decrypted);
}
