use crate::NomenKind;
use nostr_sdk::{EventId, UnsignedEvent};
use secp256k1::{schnorr::Signature, XOnlyPublicKey};

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
        .serialize())
    }

    pub fn signature_provided_op_return(&self, signature: Signature) -> Vec<u8> {
        SignatureV1 { signature }.serialize()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_op_returns() {
        let new_pubkey = XOnlyPublicKey::from_str(
            "74301b9c5d30b764bca8d3eb4febb06862f558d292fde93b4a290d90850bac91",
        )
        .unwrap();
        let tb = TransferBuilder {
            new_pubkey: &new_pubkey,
            name: "hello-world",
        };

        assert_eq!(hex::encode(tb.transfer_op_return()), "4e4f4d010174301b9c5d30b764bca8d3eb4febb06862f558d292fde93b4a290d90850bac9168656c6c6f2d776f726c64");

        // Signatures are not consistent, so they can't really be tested here.
    }
}
