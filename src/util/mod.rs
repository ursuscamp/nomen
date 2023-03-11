mod nsid;
mod pubkey;

pub use nsid::*;
pub use pubkey::*;

pub enum NamespaceNostrKind {
    Name = 38300,
    Record = 38301,
}
