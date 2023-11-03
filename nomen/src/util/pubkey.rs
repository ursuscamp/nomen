use std::{fmt::Display, str::FromStr};

use nostr_sdk::{prelude::FromPkStr, Keys};
use secp256k1::XOnlyPublicKey;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, serde_with::DeserializeFromStr)]

pub struct Pubkey(XOnlyPublicKey);

impl AsRef<XOnlyPublicKey> for Pubkey {
    fn as_ref(&self) -> &XOnlyPublicKey {
        &self.0
    }
}

impl FromStr for Pubkey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = Keys::from_pk_str(s)?;
        Ok(Pubkey(keys.public_key()))
    }
}

impl Display for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npub() {
        let _pubkey: Pubkey = "npub1u50q2x85utgcgqrmv607crvmk8x3k2nvyun84dxlj6034kajje0s2cm3r0"
            .parse()
            .unwrap();
    }

    #[test]
    fn test_hex() {
        let _pubkey: Pubkey = "e51e0518f4e2d184007b669fec0d9bb1cd1b2a6c27267ab4df969f1adbb2965f"
            .parse()
            .unwrap();
    }
}
