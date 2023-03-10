use serde::{de::IntoDeserializer, Deserialize, Serialize};

use super::ExampleDocument;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildCreate {
    pub name: String,
    pub pubkey: String,
    pub children: Vec<ChildCreate>,
}

// impl ChildCreate {
//     pub fn namespace_id(&self, parent_name: &str) -> anyhow::Result<[u; 20]> {
//         let fqdn = if parent_name.is_empty() {
//             self.name.clone()
//         } else {
//             format!("{}.{}", self.name, parent_name)
//         };

//         if self.children.is_empty() {

//         }
//     }
// }

impl ExampleDocument for ChildCreate {
    fn create_example() -> Self {
        ChildCreate {
            name: String::from("child-name"),
            pubkey: String::from("child-pubkey-hex"),
            children: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub(crate) fn namespace_id(&self) -> String {
        todo!()
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
