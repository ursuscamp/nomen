use std::path::PathBuf;

use bitcoin::Network;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Location of config file
    #[arg(short, long, default_value = ".indigo.toml")]
    pub config: Option<PathBuf>,

    /// Path for index data.
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
        #[arg(short, long)]
        bind: String,
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
        /// Minimum block confirmations for indexer
        #[arg(short, long, default_value = "3")]
        confirmations: usize,

        /// Starting block height to index. Do not specify to index from last indexed height.
        #[arg(long)]
        height: Option<usize>,
    },

    /// Query relays for missing create events.
    CreateEvents,

    /// Query relays for records
    RecordsEvents,
}
