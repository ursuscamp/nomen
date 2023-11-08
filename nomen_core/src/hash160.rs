use ripemd::{Digest, Ripemd160};
use sha2::Sha256;

#[derive(Default)]
pub struct Hash160 {
    hasher: Sha256,
}

#[allow(unused)]
impl Hash160 {
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub fn chain_update(mut self, data: &[u8]) -> Hash160 {
        self.update(data);
        self
    }

    #[allow(dead_code)]
    pub fn chain_optional(mut self, data: &Option<&[u8]>) -> Hash160 {
        if let Some(data) = data {
            self.update(data);
        }
        self
    }

    pub fn finalize(self) -> [u8; 20] {
        let f = self.hasher.finalize();
        Ripemd160::digest(f)
            .try_into()
            .expect("Hash160 struct should return 20 bytes")
    }

    pub fn fingerprint(self) -> [u8; 5] {
        let h = self.finalize();
        h[..5].try_into().unwrap()
    }

    pub fn digest(data: &[u8]) -> [u8; 20] {
        Hash160::default().chain_update(data).finalize()
    }

    pub fn digest_slices(data: &[&[u8]]) -> [u8; 20] {
        data.iter()
            .fold(Hash160::default(), |acc, d| acc.chain_update(d))
            .finalize()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_update() {
        let mut h = Hash160::default();
        h.update(b"hello");
        let d = hex::encode(h.finalize());
        assert_eq!(d, "b6a9c8c230722b7c748331a8b450f05566dc7d0f");
    }

    #[test]
    fn test_fingerprint() {
        let mut h = Hash160::default();
        h.update(b"hello");
        let d = hex::encode(h.fingerprint());
        assert_eq!(d, "b6a9c8c230");
    }

    #[test]
    fn test_digest() {
        assert_eq!(
            hex::encode(Hash160::digest(b"hello")),
            "b6a9c8c230722b7c748331a8b450f05566dc7d0f"
        );
    }

    #[test]
    fn test_digest_slices() {
        let hashed = hex::encode(Hash160::digest_slices(&[b"hello", b"world"]));
        assert_eq!(hashed, "b36c87f1c6d9182eb826d7d987f9081adf15b772");
    }
}
