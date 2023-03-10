use std::{str::FromStr, time::SystemTime};

use bitcoin::hashes::hex::ToHex;
use secp256k1::{schnorr::Signature, Message, Secp256k1, SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::pubkey::Pubkey;

pub static BROADCAST_NEW_NAME: u64 = 38300;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Event {
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u64,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl Event {
    pub fn new_broadcast_name(namespace_id: &str, name: &str) -> Event {
        Event {
            kind: BROADCAST_NEW_NAME,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("unable to calculated system time")
                .as_secs(),
            tags: vec![vec![
                "d".to_string(),
                namespace_id.to_string(),
                name.to_string(),
            ]],
            content: "[]".into(),
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> anyhow::Result<bool> {
        let id = self.id();

        // Sanity check first
        if id.to_hex() != self.id.to_ascii_lowercase() {
            return Ok(false);
        }

        // let pk = Pubkey::from_str(&self.pubkey)?;
        let pk = XOnlyPublicKey::from_str(&self.pubkey)?;
        let sig = Signature::from_str(&self.sig)?;
        let msg = Message::from_slice(&id)?;
        let secp = Secp256k1::new();
        match secp.verify_schnorr(&sig, &msg, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
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
        let id = self.id();
        self.id = id.to_hex();

        // Calculate signature
        let sig = secp.sign_schnorr(&Message::from_slice(&id)?, &kp);
        self.sig = sig.to_hex();

        Ok(self)
    }

    fn id(&self) -> [u8; 32] {
        let j = json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ]);

        let serialized = serde_json::to_string(&j).expect("Serializing event should not fail");
        println!("Serialized pre-hash: {serialized}");
        Sha256::digest(serialized)
            .try_into()
            .expect("SHA-256 should always be 32 bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        // Validate a real event
        let rawdata = r#"{
            "kind": 1,
            "content": "Nostore is submitted for review 👀",
            "tags": [],
            "created_at": 1677645192,
            "pubkey": "5cc29169f09efdfc8cf63e3458c6938f9d9d68af02d7f39d74a6882b48d7ede4",
            "id": "19bbab3c84eb921811347845150e41149d4bb4f9a4c16a5017cbfd6df0de7022",
            "sig": "ab08a9aa05de2cf7fb6fc9f25ec31d6c10d8dc69c1198ba8b86384ab425cc457e4b03452db005007d8de3538223dab0011c4cdde815c11b9c9269ca4b137784a"
          }"#;
        let event: Event = serde_json::from_str(&rawdata).unwrap();
        assert_eq!(event.is_valid().unwrap(), true);
    }

    #[test]
    fn test_is_not_valid() {
        // Validate a real event which is incorrect (same as above, kind number changed)
        let rawdata = r#"{
            "kind": 2,
            "content": "Nostore is submitted for review 👀",
            "tags": [],
            "created_at": 1677645192,
            "pubkey": "5cc29169f09efdfc8cf63e3458c6938f9d9d68af02d7f39d74a6882b48d7ede4",
            "id": "19bbab3c84eb921811347845150e41149d4bb4f9a4c16a5017cbfd6df0de7022",
            "sig": "ab08a9aa05de2cf7fb6fc9f25ec31d6c10d8dc69c1198ba8b86384ab425cc457e4b03452db005007d8de3538223dab0011c4cdde815c11b9c9269ca4b137784a"
          }"#;
        let event: Event = serde_json::from_str(&rawdata).unwrap();
        assert_eq!(event.is_valid().unwrap(), false);
    }
}
