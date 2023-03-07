use std::{ops::Sub, path::PathBuf};

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

    /// Nostr relays for commands that interact with relays.
    /// Can be specified multiple times.
    #[arg(long, short, action = clap::ArgAction::Append)]
    pub relay: Option<Vec<String>>,

    #[serde(skip)]
    #[command(subcommand)]
    pub subcommand: Subcommand,
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
            relay: self
                .relay
                .clone()
                .or(config_file.relay.clone())
                .or_else(|| {
                    Some(vec![
                        "wss://relay.damus.io".to_string(),
                        "wss://relay.snort.social".to_string(),
                    ])
                }),
            subcommand: self.subcommand.clone(),
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

#[derive(clap::Subcommand, Deserialize, Serialize, Debug, Clone)]
pub enum Subcommand {
    #[command(skip)]
    Noop,

    /// Generate a private/public keypair.
    GenerateKeypair,

    /// Create a transaction to publish a new name to the blockchain.
    NewNameTx {
        /// The top-level name to publish to the blockchain
        name: String,

        /// The input to use, in the form of "txid:vout"
        input: String,

        /// Output address for the coins
        address: String,

        /// Pubkey for name to register (in hex)
        pubkey: String,

        /// Fee rate to use for spend (in sats/vB)
        #[arg(long, default_value = "3")]
        fee_rate: usize,
    },

    /// Broadcast event for new name to Nostr relays.
    BroadcastNewName {
        /// The namespace ID for the name to broadcast.
        namespace_id: String,

        /// The private key used to the sign the nostr event. Must be the same private key that belongs to the public key used to create the name.
        privkey: String,
    },

    /// Scan and index the blockchain.
    Index,
}

impl Default for Subcommand {
    fn default() -> Self {
        Subcommand::Noop
    }
}
