use serde::{de::IntoDeserializer, Deserialize, Serialize};

use crate::{
    name::{Name, RawNameRow},
    util::Nsid,
};

use super::ExampleDocument;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChildCreate {
    pub name: String,
    pub pubkey: String,
    pub children: Vec<ChildCreate>,
}

impl ExampleDocument for ChildCreate {
    fn create_example() -> Self {
        ChildCreate {
            name: String::from("child-name"),
            pubkey: String::from("child-pubkey-hex"),
            children: vec![],
        }
    }
}

impl From<ChildCreate> for RawNameRow {
    fn from(child: ChildCreate) -> Self {
        let children: Vec<RawNameRow> = child.children.into_iter().map(|c| c.into()).collect();
        RawNameRow(child.name, child.pubkey, children)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Create {
    pub name: String,
    pub txid: String,
    pub vout: u64,
    pub address: String,
    pub pubkey: String,
    pub fee_rate: usize,
    pub children: Vec<ChildCreate>,
}

impl Create {
    pub(crate) fn namespace_id(&self) -> anyhow::Result<Nsid> {
        let name: Name = self.clone().try_into()?;
        Ok(name.namespace_id())
    }
}

impl ExampleDocument for Create {
    fn create_example() -> Self {
        Create {
            name: String::from("example-name"),
            txid: String::from("input-txid"),
            vout: 0,
            address: String::from("bc1..."),
            pubkey: String::from("pubkey hex..."),
            fee_rate: 1,
            children: vec![ChildCreate::create_example()],
        }
    }
}
