use std::time::SystemTime;

use bitcoin::hashes::hex::ToHex;
use secp256k1::{Message, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

static BROADCAST_NEW_NAME: u64 = 38300;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Event {
    id: String,
    pubkey: String,
    created_at: u64,
    kind: u64,
    tags: Vec<Vec<String>>,
    content: String,
    sig: String,
}

impl Event {
    pub fn new_broadcast_name(namespace_id: &str) -> Event {
        Event {
            kind: BROADCAST_NEW_NAME,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("unable to calculated system time")
                .as_secs(),
            tags: vec![vec!["d".to_string(), namespace_id.to_string()]],
            content: String::new(),
            ..Default::default()
        }
    }

    pub fn pubkey(mut self, pubkey: &str) -> Event {
        self.pubkey = pubkey.to_string();
        self
    }

    pub fn finalize(mut self, privkey: &str) -> anyhow::Result<Event> {
        // Write pubkey to event
        let sk = hex::decode(privkey)?;
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(&sk)?;
        let kp = sk.keypair(&secp);
        self.pubkey = sk.x_only_public_key(&secp).0.to_hex();

        // Calculate ID
        let id = self.id()?;
        self.id = id.to_hex();

        // Calculate signature
        let sig = secp.sign_schnorr(&Message::from_slice(&id)?, &kp);
        self.sig = sig.to_hex();

        Ok(self)
    }

    fn id(&self) -> anyhow::Result<[u8; 32]> {
        let j = json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ]);

        let serialized = serde_json::to_string(&j)?;
        println!("Serialized pre-hash: {serialized}");
        let id: [u8; 32] = Sha256::digest(serialized).try_into()?;

        Ok(id)
    }
}
