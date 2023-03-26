mod keyval;
mod metadata_extractor;
mod nsid;
mod nsid_builder;
mod pubkey;

pub use keyval::*;
pub use metadata_extractor::*;
pub use nsid::*;
pub use nsid_builder::*;
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
