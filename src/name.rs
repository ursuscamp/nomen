use std::str::FromStr;

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

impl TryFrom<Event> for Name {
    type Error = anyhow::Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        let pubkey = Pubkey::from_str(&event.pubkey)?;
        let d_tag = event
            .tags
            .iter()
            .find(|t| t.first().map(AsRef::as_ref) == Some("d"))
            .ok_or(anyhow!("Invalid or missing d tag"))?;
        let name = d_tag
            .iter()
            .nth(2)
            .ok_or(anyhow!("Invalid or missing d tag"))?;
        let content: Vec<RawNameRow> = serde_json::from_str(&event.content)?;
        let children: Vec<Name> = content
            .into_iter()
            .map(|rnr| rnr.try_into())
            .collect::<anyhow::Result<_>>()?;

        Ok(Name(name.clone(), pubkey, children))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RawNameRow(String, String, Vec<RawNameRow>);

impl TryFrom<RawNameRow> for Name {
    type Error = anyhow::Error;

    fn try_from(value: RawNameRow) -> Result<Self, Self::Error> {
        let pubkey = Pubkey::from_str(&value.1)?;
        let children: Vec<Name> = value
            .2
            .into_iter()
            .map(|row| row.try_into())
            .collect::<anyhow::Result<_>>()?;

        Ok(Name(value.0, pubkey, children))
    }
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

    #[test]
    fn test_raw_name_row_parse() {
        let names = r#"["hello", "5cc29169f09efdfc8cf63e3458c6938f9d9d68af02d7f39d74a6882b48d7ede4", [["world", "5cc29169f09efdfc8cf63e3458c6938f9d9d68af02d7f39d74a6882b48d7ede4", []]]]"#;
        let rows: RawNameRow = serde_json::from_str(names).unwrap();
        assert_eq!(
            rows,
            RawNameRow(
                "hello".into(),
                "5cc29169f09efdfc8cf63e3458c6938f9d9d68af02d7f39d74a6882b48d7ede4".into(),
                vec![RawNameRow(
                    "world".into(),
                    "5cc29169f09efdfc8cf63e3458c6938f9d9d68af02d7f39d74a6882b48d7ede4".into(),
                    vec![]
                )]
            )
        );
    }
}