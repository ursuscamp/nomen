use std::borrow::Cow;

use bitcoin::XOnlyPublicKey;

use crate::util::Hash160;

use super::Nsid;

pub struct NsidBuilder {
    root_name: String,
    pk: XOnlyPublicKey,
    child_hashes: Vec<Vec<u8>>,
}

impl NsidBuilder {
    pub fn new(root_name: &str, root_pk: &XOnlyPublicKey) -> NsidBuilder {
        NsidBuilder {
            root_name: root_name.to_owned(),
            pk: *root_pk,
            child_hashes: Default::default(),
        }
    }

    pub fn update_child(mut self, name: &str, pk: XOnlyPublicKey) -> Self {
        let ender = format!(".{}", self.root_name);
        let child_name = match name.ends_with(&ender) {
            true => Cow::Borrowed(name),
            false => Cow::Owned(format!("{name}.{}", self.root_name)),
        };
        let mut hasher = Hash160::default();
        hasher.update(child_name.as_bytes());
        hasher.update(&pk.serialize());
        self.child_hashes.push(hasher.finalize().to_vec());
        self
    }

    pub fn finalize(self) -> Nsid {
        let mut hasher = Hash160::default();
        hasher.update(self.root_name.as_bytes());
        if let Some(mr) = self.child_merkle_root() {
            hasher.update(&mr);
        }
        hasher.update(&self.pk.serialize());
        hasher.finalize().into()
    }

    fn child_merkle_root(&self) -> Option<Vec<u8>> {
        if self.child_hashes.is_empty() {
            return None;
        }
        let mut queue = self.child_hashes.clone();
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

        Some(queue.first().cloned().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_nsid_builder() {
        let pk: XOnlyPublicKey = "d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f"
            .parse()
            .unwrap();
        let nsid = NsidBuilder::new("smith", &pk)
            .update_child("bob", pk)
            .update_child("alice.smith", pk)
            .finalize();

        assert_eq!(
            nsid,
            "4e815dbf9d217f51ccbdfe3f24ac62a08ef8fed0".parse().unwrap()
        )
    }
}
