#![allow(unused)]

use bitcoin::{BlockHash, Txid};
use bitcoin_hashes::Hash;
use nomen_core::{self, Nsid};
use secp256k1::{schnorr::Signature, SecretKey, XOnlyPublicKey};

#[derive(serde::Deserialize, Debug)]
pub struct KeyVector {
    pub pk: XOnlyPublicKey,
    pub sk: SecretKey,
}

#[derive(serde::Deserialize, Debug)]
pub struct NsidVector {
    pub name: String,
    pub pk: XOnlyPublicKey,
    pub nsid: Nsid,
}

#[derive(serde::Deserialize, Debug)]
pub struct Hash160Vector {
    pub value: String,
    pub hash: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct NameVector {
    pub name: String,
    pub pass: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct OpReturnVector {
    pub kind: String,
    pub name: String,
    pub pk: Option<XOnlyPublicKey>,
    pub sig: Option<Signature>,
    pub value: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct TestVectors {
    pub keys: Vec<KeyVector>,
    pub hash160s: Vec<Hash160Vector>,
    pub nsids: Vec<NsidVector>,
    pub names: Vec<NameVector>,
    pub op_returns: Vec<OpReturnVector>,
}

static TEST_VECTORS: &str = include_str!("../../docs/vectors.json");

pub fn test_vectors() -> TestVectors {
    serde_json::from_str(TEST_VECTORS).unwrap()
}
