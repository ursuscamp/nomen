use std::path::PathBuf;

use anyhow::anyhow;
use bitcoin::Network;
use bitcoincore_rpc::RpcApi;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct Config {
    /// Location of config file
    #[arg(short, long, default_value = ".gun.toml")]
    #[serde(skip)]
    pub config: Option<PathBuf>,

    /// Location to Bitcoin Core cookie file.
    #[arg(long)]
    pub cookie: Option<PathBuf>,

    /// RPC username.
    #[arg(long)]
    pub rpcuser: Option<String>,

    /// RPC password.
    #[arg(long)]
    pub rpcpass: Option<String>,

    /// RPC host
    #[arg(long)]
    pub rpchost: Option<String>,

    /// RPC port number
    #[arg(long)]
    pub rpcport: Option<u16>,

    /// Bitcoin network
    #[arg(long)]
    pub network: Option<Network>,
}

impl Config {
    pub fn merge_config_file(&self, config_file: &Config) -> Config {
        Config {
            config: self.config.clone(),
            cookie: self.cookie.clone().or(config_file.cookie.clone()),
            rpcuser: self.rpcuser.clone().or(config_file.rpcuser.clone()),
            rpcpass: self.rpcpass.clone().or(config_file.rpcpass.clone()),
            rpchost: self
                .rpchost
                .clone()
                .or(config_file.rpchost.clone())
                .or(Some("localhost".into())),
            rpcport: self
                .rpcport
                .clone()
                .or(config_file.rpcport.clone())
                .or(Some(8332)),
            network: config_file
                .network
                .or(self.network)
                .or(Some(Network::Bitcoin)),
        }
    }

    pub fn rpc_client(&self) -> anyhow::Result<bitcoincore_rpc::Client> {
        let host = self.rpchost.as_ref().ok_or(anyhow!("Missing RPC host"))?;
        let port = self.rpcport.ok_or(anyhow!("Missing RPC port"))?;
        let url = format!("{host}:{port}");
        let auth = if let Some(cookie) = &self.cookie {
            bitcoincore_rpc::Auth::CookieFile(cookie.clone())
        } else if self.rpcuser.is_some() && self.rpcpass.is_some() {
            bitcoincore_rpc::Auth::UserPass(
                self.rpcuser.clone().unwrap(),
                self.rpcpass.clone().unwrap(),
            )
        } else {
            bitcoincore_rpc::Auth::None
        };
        Ok(bitcoincore_rpc::Client::new(&url, auth)?)
    }
}
