use bincode::{error::EncodeError, Decode, Encode};
use bitcoin::{hashes::hex::ToHex, BlockHash, Txid};
use std::path::Path;

use once_cell::sync::OnceCell;

use crate::name::Namespace;

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

/// Names -> NSID index
pub fn names_nsid() -> anyhow::Result<sled::Tree> {
    Ok(db().open_tree("names_nsid")?)
}

#[derive(Encode, Decode, Default, Clone)]
pub struct NamespaceModel {
    pub status: IndexStatus,
    pub name: String,
    pub nsid: Vec<u8>,
    pub prev_nsid: Option<Vec<u8>>,
    pub blockhash: Vec<u8>,
    pub txid: Vec<u8>,
    pub vout: usize,
    pub pubkey: Vec<u8>,
    pub children: Vec<Vec<u8>>,
}

impl NamespaceModel {
    pub fn new_detected(
        nsid: &str,
        prev_nsid: Option<&str>,
        blockhash: &BlockHash,
        txid: &Txid,
        vout: usize,
    ) -> anyhow::Result<NamespaceModel> {
        let nsid = hex::decode(nsid)?;
        let prev_nsid = prev_nsid.map(|s| hex::decode(s)).transpose()?;
        Ok(NamespaceModel {
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

    pub fn decode(bytes: &[u8]) -> Result<NamespaceModel, bincode::error::DecodeError> {
        let (ns, _) = bincode::decode_from_slice::<Self, _>(bytes, bincode::config::standard())?;
        Ok(ns)
    }
}

impl std::fmt::Debug for NamespaceModel {
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

#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Copy)]
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

pub fn misc_setting<D: bincode::Decode>(name: &str) -> anyhow::Result<Option<D>> {
    let misctree = db().open_tree("misc")?;
    Ok(misctree
        .get(name)?
        .map(|v| bincode::decode_from_slice::<D, _>(&v, bincode::config::standard()))
        .transpose()?
        .map(|o| o.0))
}

pub fn set_misc_setting<D: bincode::Encode>(name: &str, value: &D) -> anyhow::Result<()> {
    let misctree = db().open_tree("misc")?;
    misctree.insert(
        name,
        bincode::encode_to_vec(value, bincode::config::standard())?,
    )?;
    Ok(())
}
