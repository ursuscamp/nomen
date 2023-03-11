use std::{fmt::Debug, str::FromStr};

use derive_more::{AsMut, AsRef, Deref, DerefMut, From};

#[derive(Clone, Copy, Deref, DerefMut, AsRef, AsMut, From)]
pub struct Pubkey([u8; 32]);

impl FromStr for Pubkey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = [0u8; 32];
        hex::decode_to_slice(s, &mut out)?;
        Ok(Pubkey(out))
    }
}

impl Debug for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pubkey")
            .field(&hex::encode(&self.0))
            .finish()
    }
}
