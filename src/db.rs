use bincode::{error::EncodeError, Decode, Encode};
use bitcoin::{hashes::hex::ToHex, BlockHash, Txid};
use std::path::Path;

use once_cell::sync::OnceCell;

use crate::name::Name;

static DB: OnceCell<sled::Db> = OnceCell::new();

pub fn initialize(path: &Path) -> anyhow::Result<()> {
    DB.get_or_try_init(|| sled::open(path))?;
    Ok(())
}

pub fn db() -> sled::Db {
    DB.get().expect("db not initialized").clone()
}

pub fn flush_all() -> anyhow::Result<()> {
    namespaces()?.flush();
    Ok(())
}

pub fn namespaces() -> anyhow::Result<sled::Tree> {
    Ok(db().open_tree("namespaces")?)
}

#[derive(Encode, Decode, Default)]
pub struct Namespace {
    status: IndexStatus,
    name: String,
    nsid: Vec<u8>,
    prev_nsid: Option<Vec<u8>>,
    blockhash: Vec<u8>,
    txid: Vec<u8>,
    vout: usize,
    pubkey: Vec<u8>,
    children: Vec<Vec<u8>>,
}

impl Namespace {
    pub fn new_detected(
        nsid: &str,
        prev_nsid: Option<&str>,
        blockhash: &BlockHash,
        txid: &Txid,
        vout: usize,
    ) -> anyhow::Result<Namespace> {
        let nsid = hex::decode(nsid)?;
        let prev_nsid = prev_nsid.map(|s| hex::decode(s)).transpose()?;
        Ok(Namespace {
            nsid,
            prev_nsid,
            blockhash: blockhash.to_vec(),
            txid: txid.to_vec(),
            vout,
            ..Default::default()
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::encode_to_vec(self, bincode::config::standard())
    }

    pub fn decode(bytes: &[u8]) -> Result<Namespace, bincode::error::DecodeError> {
        let (ns, _) = bincode::decode_from_slice::<Self, _>(bytes, bincode::config::standard())?;
        Ok(ns)
    }
}

impl std::fmt::Debug for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Namespace")
            .field("status", &self.status)
            .field("name", &self.name)
            .field("nsid", &self.nsid.to_hex())
            .field("prev_nsid", &self.prev_nsid.as_ref().map(|v| v.to_hex()))
            .field("blockhash", &self.blockhash.to_hex())
            .field("txid", &self.txid.to_hex())
            .field("vout", &self.vout)
            .field("pubkey", &self.pubkey.to_hex())
            .field("children", &self.children)
            .finish()
    }
}

#[derive(Debug, Encode, Decode)]
pub enum IndexStatus {
    /// Detected on blockchain, not verified.
    Detected,

    /// Record received, not verified.
    Indexed,

    /// Index record is received, but the name is determined invalid.
    Invalid,

    /// Name valid and verified.
    Valid,

    /// Valid name with a recorded record set.
    RecordSet,
}

impl Default for IndexStatus {
    fn default() -> Self {
        IndexStatus::Detected
    }
}
