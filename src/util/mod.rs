mod extractor;
mod hash160;
mod keyval;
mod kind;
mod nsid;
mod nsid_builder;

pub use extractor::*;
pub use hash160::*;
pub use keyval::*;
pub use kind::*;
pub use nsid::*;
pub use nsid_builder::*;

pub enum NameKind {
    Name = 38300,
    Record = 38301,
    Update = 38302,
}

impl From<NameKind> for nostr_sdk::Kind {
    fn from(value: NameKind) -> Self {
        nostr_sdk::Kind::ParameterizedReplaceable(value as u16)
    }
}
