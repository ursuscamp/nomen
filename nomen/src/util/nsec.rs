use std::{fmt::Display, str::FromStr};

use nostr_sdk::{prelude::FromSkStr, Keys, ToBech32};
use secp256k1::SecretKey;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, serde_with::DeserializeFromStr)]

pub struct Nsec(SecretKey);

impl AsRef<SecretKey> for Nsec {
    fn as_ref(&self) -> &SecretKey {
        &self.0
    }
}

impl FromStr for Nsec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = Keys::from_sk_str(s)?;
        Ok(Nsec(keys.secret_key().expect("Secret key required")))
    }
}

impl Display for Nsec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_bech32().expect("Unable to format as bech32"))
    }
}

impl From<SecretKey> for Nsec {
    fn from(value: SecretKey) -> Self {
        Nsec(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nsec() {
        let _nsec: Nsec = "nsec18meshnlpsyl6qpq4jkwh9hks3v4uprp44las83akz6xfndc9tx2q646wuk"
            .parse()
            .unwrap();
    }

    #[test]
    fn test_hex() {
        let _nsec: Nsec = "3ef30bcfe1813fa00415959d72ded08b2bc08c35affb03c7b6168c99b7055994"
            .parse()
            .unwrap();
    }
}
