use std::path::PathBuf;

use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerConfigFile {
    pub bind: Option<String>,
    pub without_explorer: Option<bool>,
    pub without_api: Option<bool>,
    pub without_indexer: Option<bool>,
    pub indexer_delay: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RpcConfig {
    pub cookie: Option<PathBuf>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub network: Option<Network>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NostrConfig {
    pub relays: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ConfigFile {
    pub data: Option<PathBuf>,
    pub nostr: NostrConfig,
    pub server: ServerConfigFile,
    pub rpc: RpcConfig,
}
