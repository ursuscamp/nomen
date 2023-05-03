use std::path::PathBuf;

use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerConfigFile {
    pub bind: Option<String>,
    pub without_explorer: Option<bool>,
    pub without_api: Option<bool>,
    pub without_indexer: Option<bool>,
    pub indexer_delay: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConfigFile {
    pub data: Option<PathBuf>,
    pub cookie: Option<PathBuf>,
    pub rpcuser: Option<String>,
    pub rpcpass: Option<String>,
    pub rpchost: Option<String>,
    pub rpcport: Option<u16>,
    pub network: Option<Network>,
    pub relays: Option<Vec<String>>,
    pub server: Option<ServerConfigFile>,
}
