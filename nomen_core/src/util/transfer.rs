use crate::util::NomenKind;
use nostr_sdk::{EventId, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

use super::{SignatureV1, TransferV1};

pub struct TransferBuilder<'a> {
    pub new_pubkey: &'a XOnlyPublicKey,
    pub name: &'a str,
}

impl<'a> TransferBuilder<'a> {
    pub fn transfer_op_return(&self) -> Vec<u8> {
        TransferV1 {
            pubkey: *self.new_pubkey,
            name: self.name.to_string(),
        }
        .serialize()
    }

    pub fn unsigned_event(&self, prev_owner: &XOnlyPublicKey) -> nostr_sdk::UnsignedEvent {
        let created_at = 1u64.into();
        let kind: nostr_sdk::Kind = 1u64.into();
        let content = format!("{}{}", hex::encode(prev_owner.serialize()), self.name);
        let id = EventId::new(prev_owner, created_at, &kind, &[], &content);

        UnsignedEvent {
            id,
            pubkey: *prev_owner,
            created_at,
            kind,
            tags: vec![],
            content,
        }
    }

    pub fn signature_op_return(&self, keys: nostr_sdk::Keys) -> Result<Vec<u8>, super::UtilError> {
        let unsigned_event = self.unsigned_event(&keys.public_key());
        let event = unsigned_event.sign(&keys)?;
        Ok(SignatureV1 {
            signature: event.sig,
        }
        .seriealize())
    }
}
