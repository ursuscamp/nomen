use ripemd::{digest::OutputSizeUser, Digest, Ripemd160};
use sha2::Sha256;

#[derive(Default)]
pub struct Hash160 {
    hasher: Sha256,
}

impl Hash160 {
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub fn chain_update(mut self, data: &[u8]) -> Hash160 {
        self.update(data);
        self
    }

    pub fn finalize(self) -> [u8; 20] {
        let f = self.hasher.finalize();
        Ripemd160::digest(f)
            .try_into()
            .expect("Hash160 struct should return 20 bytes")
    }

    pub fn digest(data: &[u8]) -> [u8; 20] {
        Hash160::default().chain_update(data).finalize()
    }

    pub fn digest_slices(data: &[&[u8]]) -> [u8; 20] {
        data.into_iter()
            .fold(Hash160::default(), |acc, d| acc.chain_update(*d))
            .finalize()
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::hex::ToHex;

    use super::*;

    #[test]
    fn test_update() {
        let mut h = Hash160::default();
        h.update(b"hello");
        let d = h.finalize().to_hex();
        assert_eq!(d, "b6a9c8c230722b7c748331a8b450f05566dc7d0f");
    }

    #[test]
    fn test_digest() {
        assert_eq!(
            Hash160::digest(b"hello").to_hex(),
            "b6a9c8c230722b7c748331a8b450f05566dc7d0f"
        );
    }

    #[test]
    fn test_digest_slices() {
        let hashed = Hash160::digest_slices(&[b"hello", b"world"]).to_hex();
        assert_eq!(hashed, "b36c87f1c6d9182eb826d7d987f9081adf15b772");
    }
}
