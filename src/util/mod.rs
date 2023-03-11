mod nsid;
mod pubkey;

pub use nsid::*;
pub use pubkey::*;

pub enum NamespaceNostrKind {
    Name = 38300,
    Record = 38301,
}

impl From<NamespaceNostrKind> for nostr_sdk::Kind {
    fn from(value: NamespaceNostrKind) -> Self {
        nostr_sdk::Kind::ParameterizedReplaceable(value as u16)
    }
}
