use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    hash160::{self, Hash160},
    nostr::{Event, BROADCAST_NEW_NAME},
    nsid::Nsid,
    pubkey::Pubkey,
};

#[derive(Debug)]
pub struct Name(pub String, pub Pubkey, pub Vec<Name>);

impl Name {
    pub fn namespace_id(&self) -> Nsid {
        let mut data = self.1.to_vec();
        if self.2.is_empty() {
            data.extend(self.0.as_bytes());
            return Hash160::digest(&data).into();
        }

        let nsids: Vec<_> = self.2.iter().map(Name::namespace_id).collect();
        let mr = merkle_root(&nsids);
        data.extend(mr.as_ref());
        data.extend(self.0.as_bytes());
        Hash160::digest(&data).into()
    }
}

fn merkle_root(ids: &[Nsid]) -> Nsid {
    let mut queue = ids.to_vec();
    if queue.len() % 2 != 0 {
        queue.push(
            queue
                .last()
                .cloned()
                .expect("merkle_root expects at least one item"),
        );
    }

    while queue.len() > 1 {
        queue = queue
            .chunks(2)
            .map(|chunk| Hash160::digest_slices(&[chunk[0].as_ref(), chunk[1].as_ref()]).into())
            .collect();
    }

    queue.first().copied().unwrap()
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::hex::ToHex;

    use super::*;

    #[test]
    fn test_merkle_root() {
        let nsids = vec![[0u8; 20].into(), [1; 20].into(), [2; 20].into()];
        let mr = merkle_root(&nsids);
        assert_eq!(mr.to_hex(), "3e85acc67048cc0a3e9a333a59f529f81b71c36f");
    }

    #[test]
    fn test_namespace_id() {
        let name = Name(
            "com".into(),
            [0; 32].into(),
            vec![
                Name("amazon.com".into(), [0; 32].into(), vec![]),
                Name("google.com".into(), [0; 32].into(), vec![]),
            ],
        );
        let nsid = name.namespace_id();
        assert_eq!(nsid.to_hex(), "8c78c5b6accdd573f3b061abe61e441ed792a550");
    }
}
