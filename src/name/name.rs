use std::collections::HashMap;

use ripemd::{Digest, Ripemd160};
use sha2::Sha256;

use crate::hash160::Hash160;

use self::util::merkle_root;

use super::Validator;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Name {
    pub parent_name: String,
    pub name: String,
    pub pubkey: [u8; 32],
    pub names: Vec<Name>,
}

impl Name {
    fn namespace_id(&self) -> [u8; 20] {
        let mr = self.merkle_root();
        let slice = mr.iter().map(|d| d.as_slice()).next();
        Hash160::default()
            .chain_update(&self.pubkey)
            .chain_optional(&slice)
            .chain_update(self.fqdn().as_bytes())
            .finalize()
    }

    fn merkle_root(&self) -> Option<[u8; 20]> {
        let mut nr = self.names_roots();
        merkle_root(nr)
    }

    fn names_roots(&self) -> Vec<[u8; 20]> {
        self.names.iter().fold(Vec::new(), |mut acc, name| {
            acc.push(name.namespace_id());
            acc
        })
    }

    pub fn fqdn(&self) -> String {
        if self.parent_name.len() > 0 {
            return [self.name.as_str(), self.parent_name.as_str()].join(".");
        }
        self.name.clone()
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        Validator::new(self).validate()
    }
}

mod util {
    use ripemd::{Digest, Ripemd160};
    use sha2::Sha256;

    use crate::hash160::Hash160;

    pub fn merkle_root(mut hashes: Vec<[u8; 20]>) -> Option<[u8; 20]> {
        if hashes.is_empty() {
            return None;
        }

        // If the number is odd, then we need to make it even by putting the last one twice
        if hashes.len() % 2 != 0 {
            let last = hashes.last().unwrap().clone();
            hashes.push(last);
        }

        while hashes.len() > 1 {
            hashes = merkle_round(hashes);
        }

        hashes.get(0).cloned()
    }

    fn merkle_round(hashes: Vec<[u8; 20]>) -> Vec<[u8; 20]> {
        hashes
            .chunks(2)
            .map(|chunk| {
                let first = chunk[0];
                let second = chunk.get(1).unwrap_or(&first);
                Hash160::digest_slices(&[&first, second])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::hex::ToHex;

    use super::{util::merkle_root, *};

    #[test]
    fn test_merkle_root() {
        let hashes = vec![
            Hash160::digest(b"hello"),
            Hash160::digest(b"world"),
            Hash160::digest(b"!"),
        ];

        let root = merkle_root(hashes);
        assert_eq!(
            root.unwrap().to_hex(),
            "ccc5697822f504f001381874baabe420e7fd63e0"
        );
    }

    #[test]
    fn test_fqdn() {
        let name = new_name();
        assert_eq!(name.fqdn(), "com");
        assert_eq!(name.names[0].fqdn(), "amazon.com");
    }

    #[test]
    fn test_namespace_id() {
        let name = new_name();

        assert_eq!(
            name.names[0].namespace_id().to_hex(),
            "89a7a460cef73e8b9542488e7cab7ebfe888972a"
        );
        assert_eq!(
            name.merkle_root().unwrap().to_hex(),
            "db05eb8c5a0612fd2583c4d8318600a3d67035e3"
        );

        assert_eq!(
            name.namespace_id().to_hex(),
            "e3384992f1c5f1551e71c3cc11ecaf9070cf5fe6"
        );
    }

    fn new_name() -> Name {
        Name {
            parent_name: String::new(),
            name: "com".to_string(),
            pubkey: [0; 32],
            names: vec![Name {
                parent_name: "com".into(),
                name: "amazon".into(),
                pubkey: [1; 32],
                names: vec![],
            }],
        }
    }
}
