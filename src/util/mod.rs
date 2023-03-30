mod childpair;
mod extractor;
mod hash160;
mod indigo_tx;
mod keyval;
mod nsid;
mod nsid_builder;

use anyhow::{anyhow, bail};
use bitcoin::Block;
pub use childpair::*;
pub use extractor::*;
pub use hash160::*;
pub use indigo_tx::*;
pub use keyval::*;
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
