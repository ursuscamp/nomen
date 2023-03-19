use std::str::FromStr;

use anyhow::anyhow;
use nostr_sdk::{prelude::TagKind, Event, Tag};
use serde::{Deserialize, Serialize};

use crate::{
    documents::{ChildCreate, Create},
    hash160::Hash160,
    util::{Nsid, Pubkey},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace(pub String, pub Pubkey, pub Vec<Namespace>);

impl Namespace {
    pub fn namespace_id(&self) -> Nsid {
        self.namespace_id_("")
    }

    fn namespace_id_(&self, parent: &str) -> Nsid {
        let fqdn = if parent.is_empty() {
            self.0.clone()
        } else {
            format!("{}.{}", self.0, parent)
        };
        let mut data = self.1.to_vec();
        if self.2.is_empty() {
            data.extend(fqdn.as_bytes());
            return Hash160::digest(&data).into();
        }

        let nsids: Vec<_> = self.2.iter().map(|n| n.namespace_id_(&fqdn)).collect();
        let mr = merkle_root(&nsids);
        data.extend(mr.as_ref());

        data.extend(fqdn.as_bytes());
        Hash160::digest(&data).into()
    }
}

impl TryFrom<Event> for Namespace {
    type Error = anyhow::Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        let nsid: Nsid = get_d_tag(&event)
            .ok_or_else(|| anyhow!("Missing d tag"))?
            .parse()?;
        let name = get_ind_tag(&event)
            .ok_or_else(|| anyhow!("Missing ind tag"))?
            .clone();
        let children = get_names(&event)?;
        let pubkey = event.pubkey.into();
        let namespace = Namespace(name, pubkey, children);

        if namespace.namespace_id() != nsid {
            return Err(anyhow!("Invalid namespace id"));
        }

        Ok(namespace)
    }
}

impl TryFrom<Create> for Namespace {
    type Error = anyhow::Error;

    fn try_from(value: Create) -> Result<Self, Self::Error> {
        Ok(Namespace(
            value.name.clone(),
            Pubkey::from_str(&value.pubkey)?,
            value
                .children
                .into_iter()
                .map(|child| child.try_into())
                .collect::<anyhow::Result<_>>()?,
        ))
    }
}

impl TryFrom<ChildCreate> for Namespace {
    type Error = anyhow::Error;

    fn try_from(value: ChildCreate) -> Result<Self, Self::Error> {
        Ok(Namespace(
            value.name.clone(),
            Pubkey::from_str(&value.pubkey)?,
            value
                .children
                .into_iter()
                .map(|child| child.try_into())
                .collect::<anyhow::Result<_>>()?,
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawNameRow(pub String, pub String, pub Vec<RawNameRow>);

impl TryFrom<RawNameRow> for Namespace {
    type Error = anyhow::Error;

    fn try_from(value: RawNameRow) -> Result<Self, Self::Error> {
        let pubkey = Pubkey::from_str(&value.1)?;
        let children: Vec<Namespace> = value
            .2
            .into_iter()
            .map(|row| row.try_into())
            .collect::<anyhow::Result<_>>()?;

        Ok(Namespace(value.0, pubkey, children))
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

fn get_d_tag(event: &Event) -> Option<&String> {
    event.tags.iter().find_map(|t| match t {
        Tag::Identifier(id) => Some(id),
        _ => None,
    })
}

fn get_ind_tag(event: &Event) -> Option<&String> {
    event.tags.iter().find_map(|t| match t {
        Tag::Generic(TagKind::Custom(tagname), strs) if tagname == "ind" => strs.first(),
        _ => None,
    })
}

fn get_names(event: &Event) -> anyhow::Result<Vec<Namespace>> {
    let children: Vec<ChildCreate> = serde_json::from_str(&event.content)?;
    children
        .into_iter()
        .map(|cc| cc.try_into())
        .collect::<anyhow::Result<_>>()
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
        let name = Namespace(
            "com".into(),
            [0; 32].into(),
            vec![
                Namespace("amazon".into(), [0; 32].into(), vec![]),
                Namespace("google".into(), [0; 32].into(), vec![]),
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
