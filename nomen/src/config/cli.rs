use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Location of config file: Default: nomen.toml
    #[arg(short, long, default_value = "nomen.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Subcommand {
    /// Output example config file.
    Init,

    /// Scan and index the blockchain.
    Index,

    /// Start the HTTP server
    Server,
}