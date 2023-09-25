use nomen_core::util::NomenKind;
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

    pub fn signature_op_return(&self, keys: nostr_sdk::Keys) -> anyhow::Result<Vec<u8>> {
        let pubkey = keys.public_key();
        let created_at = 1u64.into();
        let kind: nostr_sdk::Kind = 1u64.into();
        let content = format!("{}{}", hex::encode(pubkey.serialize()), self.name);
        let id = EventId::new(&pubkey, created_at, &kind, &[], &content);
        let unsigned_event = UnsignedEvent {
            id,
            pubkey,
            created_at,
            kind,
            tags: vec![],
            content,
        };
        let event = unsigned_event.sign(&keys)?;
        let v: Vec<u8> = b"NOM 0x01 0x02"
            .to_vec()
            .iter()
            .chain(event.sig.as_ref().iter())
            .copied()
            .collect();
        Ok(v)
    }
}
