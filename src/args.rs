use std::path::PathBuf;

use bitcoin::Network;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct Args {
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
    #[arg(long, default_value = "8332")]
    pub rpcport: Option<u16>,

    /// Bitcoin network
    #[arg(long, default_value = "bitcoin")]
    pub network: Option<Network>,
}
impl Args {
    pub fn merge(&self, config: &Args) -> Args {
        Args {
            config: config.config.clone().or(self.config.clone()),
            cookie: config.cookie.clone().or(self.cookie.clone()),
            rpcuser: config.rpcuser.clone().or(self.rpcuser.clone()),
            rpcpass: config.rpcpass.clone().or(self.rpcpass.clone()),
            rpchost: config.rpchost.clone().or(self.rpchost.clone()),
            rpcport: config.rpcport.or(self.rpcport).or(Some(8443)),
            network: config.network.or(self.network).or(Some(Network::Bitcoin)),
        }
    }
}
