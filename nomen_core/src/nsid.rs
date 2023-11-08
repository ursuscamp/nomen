use std::{
    fmt::{Debug, Display},
    io::Read,
    str::FromStr,
};

use bitcoin::secp256k1::XOnlyPublicKey;
use derive_more::{AsMut, AsRef, Deref, DerefMut, From};
use nostr_sdk::Event;

use super::{EventExtractor, NameKind, NsidBuilder};

#[derive(
    Clone, Copy, Deref, DerefMut, AsRef, AsMut, From, Eq, PartialEq, serde_with::DeserializeFromStr,
)]
pub struct Nsid([u8; 20]);

impl Nsid {
    #[allow(dead_code)]
    pub fn from_slice(bytes: &[u8]) -> Result<Nsid, super::UtilError> {
        Ok(Nsid(bytes.try_into()?))
    }
}

impl TryFrom<&[u8]> for Nsid {
    type Error = super::UtilError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Nsid::from_slice(value)
    }
}

impl TryFrom<Event> for Nsid {
    type Error = super::UtilError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        let nk: NameKind = event.kind.try_into()?;
        let name = event.extract_name()?;
        let builder = match nk {
            NameKind::Name => NsidBuilder::new(&name, &event.pubkey),
        };
        Ok(builder.finalize())
    }
}

impl FromStr for Nsid {
    type Err = super::UtilError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = [0u8; 20];
        hex::decode_to_slice(s, &mut out)?;
        Ok(Nsid(out))
    }
}

impl Debug for Nsid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pubkey").field(&hex::encode(self.0)).finish()
    }
}

impl Display for Nsid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}
