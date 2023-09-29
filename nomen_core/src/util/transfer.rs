use crate::util::NomenKind;
use nostr_sdk::{EventId, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

pub struct TransferBuilder<'a> {
    pub new: &'a XOnlyPublicKey,
    pub name: &'a str,
}

impl<'a> TransferBuilder<'a> {
    pub fn transfer_op_return(&self) -> Vec<u8> {
        b"NOM\x01\x01"
            .iter()
            .chain(self.new.serialize().iter())
            .chain(self.name.as_bytes().iter())
            .copied()
            .collect()
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
        let v: Vec<u8> = b"NOM\x01\x02"
            .to_vec()
            .iter()
            .chain(event.sig.as_ref().iter())
            .copied()
            .collect();
        Ok(v)
    }
}
