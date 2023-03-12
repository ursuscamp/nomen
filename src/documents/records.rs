use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ExampleDocument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Records {
    pub name: String,
    pub records: HashMap<String, String>,
}

impl ExampleDocument for Records {
    fn create_example() -> Self {
        Records {
            name: "example.com".into(),
            records: [
                ("IP4".into(), "127.0.0.1".into()),
                ("IP6".into(), "::1".into()),
                ("NPUB".into(), "npub1...".into()),
            ]
            .into(),
        }
    }
}
