use std::str::FromStr;

use derive_more::{AsRef, From, Into};
use nostr_sdk::{
    prelude::{FromPkStr, FromSkStr},
    Keys,
};
use secp256k1::{SecretKey, XOnlyPublicKey};

#[derive(Debug, Clone, PartialEq, Eq, From, Into, AsRef)]
pub struct Nsec(SecretKey);

impl FromStr for Nsec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = Keys::from_sk_str(s)?;
        let sk = keys.secret_key()?;
        Ok(Nsec(sk))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, From, Into, AsRef)]
pub struct Npub(XOnlyPublicKey);

impl FromStr for Npub {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = Keys::from_pk_str(s)?;
        let pk = keys.public_key();
        Ok(Npub(pk))
    }
}
