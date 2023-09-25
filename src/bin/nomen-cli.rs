use clap::Parser;
use nostr_sdk::ToBech32;

pub fn main() -> anyhow::Result<()> {
    let ops = Ops::parse();

    handle_ops(ops)?;

    Ok(())
}

fn handle_ops(ops: Ops) -> anyhow::Result<()> {
    match ops.command {
        Commands::Keys { pubkey, nostr } => cmd_keys(pubkey, nostr)?,
    }

    Ok(())
}

fn cmd_keys(pubkey: bool, nostr: bool) -> anyhow::Result<()> {
    let keys = nostr_sdk::Keys::generate();
    let (sk, pk) = if nostr {
        (
            keys.secret_key()?.to_bech32()?,
            keys.public_key().to_bech32()?,
        )
    } else {
        (
            keys.secret_key()?.display_secret().to_string(),
            keys.public_key().to_string(),
        )
    };
    println!("SK: {sk}");
    if pubkey {
        println!("PK: {pk}");
    }
    Ok(())
}

#[derive(clap::Parser)]
struct Ops {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate Schnorr keypairs.
    Keys {
        #[arg(short, long)]
        pubkey: bool,

        #[arg(short, long)]
        nostr: bool,
    },
}
