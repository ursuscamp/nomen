use std::collections::HashMap;

use ripemd::{Digest, Ripemd160};
use sha2::Sha256;

use crate::hash160::Hash160;

#[derive(Debug, PartialEq, Eq)]
pub struct Name {
    pub name: String,
    pub pubkey: [u8; 32],
    pub meta: HashMap<String, String>,
    pub names: Vec<Name>,
}

impl Name {
    pub fn namespace_id(&self) -> [u8; 20] {
        Hash160::default()
            .chain_update(&self.pubkey)
            .chain_update(&self.merkle_root())
            .chain_update(self.name.as_bytes())
            .finalize()
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

    use crate::hash160::Hash160;

    pub fn merkle_round(hashes: Vec<[u8; 20]>) -> Vec<[u8; 20]> {
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

    use super::{util::merkle_round, *};

    #[test]
    fn test_merkle_round() {
        let round0 = vec![
            Hash160::digest(b"hello"),
            Hash160::digest(b"world"),
            Hash160::digest(b"!"),
        ];
        let round1 = merkle_round(round0.clone());
        assert_eq!(round1[0], Hash160::digest_slices(&[&round0[0], &round0[1]]));
        assert_eq!(round1[1], Hash160::digest_slices(&[&round0[2], &round0[2]]));

        let round2 = merkle_round(round1.clone());
        assert_eq!(round2.len(), 1);
        assert_eq!(round2[0], Hash160::digest_slices(&[&round1[0], &round1[1]]));
        assert_eq!(
            round2[0].to_hex(),
            "ccc5697822f504f001381874baabe420e7fd63e0"
        );
    }
}
