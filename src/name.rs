use std::collections::HashMap;

use ripemd::{Digest, Ripemd160};
use sha2::Sha256;

type RipemdHash = [u8; 20];
type Sha256Hash = [u8; 32];

#[derive(Debug, PartialEq, Eq)]
pub struct Name {
    pub name: String,
    pub pubkey: [u8; 32],
    pub meta: HashMap<String, String>,
    pub names: Vec<Name>,
}

impl Name {
    pub fn namespace_id(&self) -> [u8; 20] {
        // Calculate the SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&self.pubkey);
        hasher.update(&self.merkle_root());
        hasher.update(&self.name);
        let sha = hasher.finalize();

        // Calc the RIPEMD-160
        let mut hasher = Ripemd160::new();
        hasher.update(sha);
        hasher.finalize().try_into().unwrap()
    }

    fn merkle_root(&self) -> [u8; 20] {
        let mut nr = self.names_roots();
        while nr.len() > 1 {
            nr = util::merkle_round(nr);
        }

        nr[0]
    }

    fn names_roots(&self) -> Vec<[u8; 20]> {
        self.names.iter().fold(Vec::new(), |mut acc, name| {
            acc.push(name.namespace_id());
            acc
        })
    }
}

mod util {
    use ripemd::{Digest, Ripemd160};
    use sha2::Sha256;

    use super::RipemdHash;

    pub fn merkle_round(hashes: Vec<RipemdHash>) -> Vec<RipemdHash> {
        hashes
            .chunks(2)
            .map(|chunk| {
                let first = chunk[0];
                let second = chunk.get(1).unwrap_or(&first);
                hash160_slice(&[&first, second])
            })
            .collect()
    }

    pub fn hash160(bytes: &[u8]) -> RipemdHash {
        Ripemd160::digest(Sha256::digest(bytes))
            .try_into()
            .expect("hash160 should always return 20 bytes")
    }

    pub fn hash160_slice(bytes: &[&[u8]]) -> RipemdHash {
        let sha = bytes
            .into_iter()
            .fold(Sha256::new(), |acc, b| acc.chain_update(*b))
            .finalize();
        let mut hasher = Ripemd160::new();
        hasher.update(sha);
        hasher
            .finalize()
            .try_into()
            .expect("hash160_slice should return 20 bytes")
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::hex::ToHex;

    use crate::name::util::hash160_slice;

    use super::{util::merkle_round, *};

    #[test]
    fn test_hash160() {
        let hashed = util::hash160(b"hello");
        let hex = hashed.to_hex();
        assert_eq!(hex, "b6a9c8c230722b7c748331a8b450f05566dc7d0f");
    }

    #[test]
    fn test_hash160_slice() {
        let hashed = util::hash160_slice(&[b"hello", b"world"]).to_hex();
        assert_eq!(hashed, "b36c87f1c6d9182eb826d7d987f9081adf15b772");
    }

    #[test]
    fn test_merkle_round() {
        let round0 = vec![
            util::hash160(b"hello"),
            util::hash160(b"world"),
            util::hash160(b"!"),
        ];
        let round1 = merkle_round(round0.clone());
        assert_eq!(round1[0], util::hash160_slice(&[&round0[0], &round0[1]]));
        assert_eq!(round1[1], util::hash160_slice(&[&round0[2], &round0[2]]));

        let round2 = merkle_round(round1.clone());
        assert_eq!(round2.len(), 1);
        assert_eq!(round2[0], hash160_slice(&[&round1[0], &round1[1]]));
        assert_eq!(
            round2[0].to_hex(),
            "ccc5697822f504f001381874baabe420e7fd63e0"
        );
    }
}
