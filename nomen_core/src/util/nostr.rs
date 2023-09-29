use std::str::FromStr;

use derive_more::{AsRef, From, Into};
use nostr_sdk::{
    prelude::{FromPkStr, FromSkStr},
    Keys,
};
use secp256k1::{SecretKey, XOnlyPublicKey};

#[derive(Debug, Clone, PartialEq, Eq, From, Into, AsRef)]
pub struct NostrSk(SecretKey);

impl FromStr for NostrSk {
    type Err = super::UtilError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = Keys::from_sk_str(s)?;
        let sk = keys.secret_key()?;
        Ok(NostrSk(sk))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, From, Into, AsRef)]
pub struct NostrPk(XOnlyPublicKey);

impl FromStr for NostrPk {
    type Err = super::UtilError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = Keys::from_pk_str(s)?;
        let pk = keys.public_key();
        Ok(NostrPk(pk))
    }
}
