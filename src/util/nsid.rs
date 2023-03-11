use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use anyhow::anyhow;
use bitcoin::hashes::hex::ToHex;
use derive_more::{AsMut, AsRef, Deref, DerefMut, From};

#[derive(Clone, Copy, Deref, DerefMut, AsRef, AsMut, From, Eq, PartialEq)]
pub struct Nsid([u8; 20]);

impl FromStr for Nsid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = [0u8; 20];
        hex::decode_to_slice(s, &mut out)?;
        Ok(Nsid(out))
    }
}

impl Debug for Nsid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pubkey")
            .field(&hex::encode(&self.0))
            .finish()
    }
}

impl Display for Nsid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}
