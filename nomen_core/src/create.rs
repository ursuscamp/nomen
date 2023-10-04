use crate::NomenKind;
use nostr_sdk::{EventId, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

use super::{CreateV0, CreateV1, Hash160, NsidBuilder};

pub struct CreateBuilder<'a> {
    pub pubkey: &'a XOnlyPublicKey,
    pub name: &'a str,
}

impl<'a> CreateBuilder<'a> {
    pub fn new(pubkey: &'a XOnlyPublicKey, name: &'a str) -> CreateBuilder<'a> {
        CreateBuilder { pubkey, name }
    }

    pub fn v0_op_return(&self) -> Vec<u8> {
        let fingerprint = Hash160::default()
            .chain_update(self.name.as_bytes())
            .fingerprint();
        let nsid = NsidBuilder::new(self.name, self.pubkey).finalize();
        CreateV0 { fingerprint, nsid }.serialize()
    }

    pub fn v1_op_return(&self) -> Vec<u8> {
        CreateV1 {
            pubkey: *self.pubkey,
            name: self.name.to_string(),
        }
        .serialize()
    }
}
