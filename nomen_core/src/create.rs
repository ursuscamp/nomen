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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_op_returns() {
        let pk = "60de6fbc4a78209942c62706d904ff9592c2e856f219793f7f73e62fc33bfc18"
            .parse()
            .unwrap();
        let cb = CreateBuilder::new(&pk, "hello-world");

        assert_eq!(
            hex::encode(cb.v0_op_return()),
            "4e4f4d0000e5401df4b4273968a1e7be2ef0acbcae6f61d53e73101e2983"
        );

        assert_eq!(hex::encode(cb.v1_op_return()), "4e4f4d010060de6fbc4a78209942c62706d904ff9592c2e856f219793f7f73e62fc33bfc1868656c6c6f2d776f726c64");
    }
}
