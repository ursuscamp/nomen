mod childpair;
mod hash160;
mod keyval;
mod nsid;
mod nsid_builder;

pub use childpair::*;
pub use hash160::*;
pub use keyval::*;
pub use nsid::*;
pub use nsid_builder::*;

pub enum NamespaceNostrKind {
    Name = 38300,
    Record = 38301,
}

impl From<NamespaceNostrKind> for nostr_sdk::Kind {
    fn from(value: NamespaceNostrKind) -> Self {
        nostr_sdk::Kind::ParameterizedReplaceable(value as u16)
    }
}
