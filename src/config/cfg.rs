use std::path::PathBuf;

use bitcoin::Network;
use clap::Parser;
use nostr_sdk::{
    prelude::{FromSkStr, ToBech32},
    Options,
};
use ripemd::digest::Update;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite, SqlitePool};

use super::ConfigFile;

#[derive(Parser, Debug, Clone)]
pub struct Config {
    /// Location of config file: Default: .indigo.toml
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Path for index data. Default: indigo.db
    #[arg(short, long)]
    pub data: Option<PathBuf>,

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
    pub relays: Option<Vec<String>>,

    #[command(subcommand)]
    pub subcommand: Subcommand,
}

impl Config {
    pub fn merge_config_file(&mut self, cf: ConfigFile) {
        self.data = self
            .data
            .take()
            .or(cf.data)
            .or_else(|| Some("indigo.db".into()));
        self.cookie = self.cookie.take().or(cf.cookie);
        self.rpcuser = self.rpcuser.take().or(cf.rpcuser);
        self.rpcpass = self.rpcpass.take().or(cf.rpcpass);
        self.rpchost = self.rpchost.take().or(cf.rpchost);
        self.rpcport = self.rpcport.or(cf.rpcport);
        self.network = self.network.or(cf.network);
        self.relays = self.relays.take().or(cf.relays);
        if let Subcommand::Server {
            bind,
            confirmations,
            height,
            without_explorer,
            without_api,
            without_indexer,
        } = &mut self.subcommand
        {
            let mut server = cf.server.unwrap_or_default();
            *bind = bind
                .take()
                .or_else(|| server.bind.take())
                .or_else(|| Some("0.0.0.0:8080".into()));
            *confirmations = confirmations
                .take()
                .or_else(|| server.confirmations.take())
                .or(Some(3));
            *height = height.take().or_else(|| server.height.take()).or(Some(1));
            *without_explorer = *without_explorer || server.without_explorer.unwrap_or_default();
            *without_api = *without_api || server.without_api.unwrap_or_default();
            *without_indexer = *without_indexer || server.without_indexer.unwrap_or_default();
        }
    }

    pub fn rpc_auth(&self) -> bitcoincore_rpc::Auth {
        if let Some(cookie) = &self.cookie {
            bitcoincore_rpc::Auth::CookieFile(cookie.clone())
        } else if self.rpcuser.is_some() || self.rpcpass.is_some() {
            bitcoincore_rpc::Auth::UserPass(
                self.rpcuser.clone().expect("RPC user not configured"),
                self.rpcpass.clone().expect("RPC password not configured"),
            )
        } else {
            bitcoincore_rpc::Auth::None
        }
    }

    pub fn server_bind(&self) -> Option<String> {
        match &self.subcommand {
            Subcommand::Server { bind, .. } => bind.clone(),
            _ => None,
        }
    }

    pub fn server_confirmations(&self) -> Option<usize> {
        match &self.subcommand {
            Subcommand::Server { confirmations, .. } => *confirmations,
            _ => None,
        }
    }

    pub fn server_height(&self) -> Option<usize> {
        match &self.subcommand {
            Subcommand::Server { height, .. } => *height,
            _ => None,
        }
    }

    pub fn rpc_client(&self) -> anyhow::Result<bitcoincore_rpc::Client> {
        let host = self.rpchost.clone().unwrap_or_else(|| "127.0.0.1".into());
        let port = self.rpcport.unwrap_or(8332);
        let url = format!("{host}:{port}");
        let auth = self.rpc_auth();
        Ok(bitcoincore_rpc::Client::new(&url, auth)?)
    }

    pub async fn sqlite(&self) -> anyhow::Result<sqlite::SqlitePool> {
        let db = self.data.clone().expect("No database configured");

        // SQLx doesn't seem to like it if a db file does not already exist, so let's create an empty one
        if !tokio::fs::try_exists(&db).await? {
            tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&db)
                .await?;
        }

        let db = self
            .data
            .as_ref()
            .and_then(|d| d.to_str().map(|s| s.to_owned()))
            .expect("No database configured");

        Ok(SqlitePool::connect(&format!("sqlite:{db}")).await?)
    }

    pub async fn nostr_client(
        &self,
        sk: &str,
    ) -> anyhow::Result<(nostr_sdk::Keys, nostr_sdk::Client)> {
        let keys = nostr_sdk::Keys::from_sk_str(sk)?;
        let client = nostr_sdk::Client::new_with_opts(&keys, Options::new().wait_for_send(true));
        let relays = self.relays.as_ref().expect("No relays configured");
        for relay in relays {
            client.add_relay(relay, None).await?;
        }
        client.connect().await;
        Ok((keys, client))
    }

    pub async fn nostr_random_client(
        &self,
    ) -> anyhow::Result<(nostr_sdk::Keys, nostr_sdk::Client)> {
        let keys = nostr_sdk::Keys::generate();
        let sk = keys.secret_key()?.to_bech32()?;
        self.nostr_client(&sk).await
    }
}

#[derive(clap::Subcommand, Deserialize, Serialize, Debug, Clone)]
pub enum Subcommand {
    #[command(skip)]
    Noop,

    /// Generate a private/public keypair.
    GenerateKeypair,

    /// Create and broadcast new names.
    #[command(subcommand)]
    #[serde(skip)]
    New(NewSubcommand),

    /// Create and broadcast updates.
    #[command(subcommand)]
    #[serde(skip)]
    Update(UpdateSubcommand),

    /// Broadcast records updates
    #[command(subcommand)]
    #[serde(skip)]
    Records(RecordsSubcommand),

    /// Scan and index the blockchain.
    #[command(subcommand)]
    #[serde(skip)]
    Index(IndexSubcommand),

    /// Useful debugging commands
    #[command(subcommand)]
    #[serde(skip)]
    Debug(DebugSubcommand),

    /// Start the HTTP server
    Server {
        /// Address and port to bind.
        #[arg(short, long)]
        bind: Option<String>,

        /// Minimum number of confirmation before adding to index. Default: 3
        #[arg(long)]
        confirmations: Option<usize>,

        /// Starting block height to index. Default: most recently scanned block
        #[arg(long)]
        height: Option<usize>,

        /// Start server without explorer.
        #[arg(long)]
        without_explorer: bool,

        /// Start server without API.
        #[arg(long)]
        without_api: bool,

        /// Start server without indexer.
        #[arg(long)]
        without_indexer: bool,
    },
}

impl Default for Subcommand {
    fn default() -> Self {
        Subcommand::Noop
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum NewSubcommand {
    /// Create a new, unsigned transaction using a simple input document.
    /// Use `indigo new example` to create a sample document.
    Tx { document: PathBuf },

    /// Broadcast the new name transaction to Nostr relays.
    Broadcast {
        /// The same document used to create the name.
        document: PathBuf,

        /// Private key to sign the Nostr event
        privkey: String,
    },

    /// Print an example document for new names.
    Example,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum RecordsSubcommand {
    /// Broadcast a records event to the relays.
    Broadcast {
        /// A document representing the records to be relayed.
        document: PathBuf,

        /// Private key to sign the Nostr event.
        privkey: String,
    },

    /// Print an example document to update the name.
    Example,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum DebugSubcommand {
    ListNamespaces,
    NamesIndex,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum IndexSubcommand {
    /// Index the blockchain and look for new namespaces.
    Blockchain {
        /// Minimum block confirmations for indexer. Default: 3
        #[arg(short, long)]
        confirmations: Option<usize>,

        /// Starting block height to index. Default: most recently scanned block
        #[arg(long)]
        height: Option<usize>,
    },

    /// Query relays for missing create events.
    CreateEvents,

    /// Query relays for records
    RecordsEvents,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum UpdateSubcommand {
    /// Create a new, unsigned transaction using a simple input document.
    /// Use `indigo update example` to create a sample document.
    Tx { document: PathBuf },

    /// Broadcast the update name transaction to Nostr relays.
    Broadcast {
        /// The same document used to create the name.
        document: PathBuf,

        /// Private key to sign the Nostr event
        privkey: String,
    },

    /// Print an example document for update names.
    Example,
}
