use bitcoin::hashes::hex::ToHex;
use secp256k1::Secp256k1;
use yansi::Paint;

pub fn generate_keypair() {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
    let (public_key, _) = public_key.x_only_public_key();

    let secret_key = secret_key.secret_bytes().to_hex();
    let public_key = public_key.serialize().to_hex();

    println!("{}{}", Paint::red("Secret Key: "), secret_key);
    println!("{}{}", Paint::green("Public Key: "), public_key);
}
