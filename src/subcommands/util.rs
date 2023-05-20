use std::{io::Write, path::PathBuf};

use anyhow::bail;
use nostr_sdk::UnsignedEvent;
use secp256k1::{Secp256k1, XOnlyPublicKey};
use yansi::Paint;

use crate::{
    config::{Config, ConfigFile, SignEventCommand},
    util::{check_name_availability, Hash160, NomenKind, NsidBuilder},
};

use super::get_keys;

pub fn generate_keypair() {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
    let (public_key, _) = public_key.x_only_public_key();

    let secret_key = hex::encode(secret_key.secret_bytes());
    let public_key = hex::encode(public_key.serialize());

    println!("{}{}", Paint::red("Secret Key: "), secret_key);
    println!("{}{}", Paint::green("Public Key: "), public_key);
}

pub async fn lookup(config: &Config, name: &str) -> anyhow::Result<()> {
    let name = name.to_lowercase();
    let (name, msg) = match check_name_availability(config, &name).await {
        Ok(_) => (Paint::yellow(&name), Paint::green("available")),
        Err(_) => (Paint::yellow(&name), Paint::red("unavailable")),
    };

    println!("Name {name} is {msg}.");
    Ok(())
}

pub fn init_config(path: &Option<PathBuf>) -> anyhow::Result<()> {
    let file = path.clone().unwrap_or_else(|| ".nomen.toml".into());
    if file.exists() {
        bail!("Config file already exists.");
    }

    let mut file = std::fs::File::create(&file)?;
    let config_file = ConfigFile::init();

    let strout = toml::to_string_pretty(&config_file)?;
    file.write_all(strout.as_bytes())?;
    Ok(())
}

pub async fn sign_event(config: &Config, args: &SignEventCommand) -> anyhow::Result<()> {
    let keys = get_keys(&args.privkey)?;
    let event: UnsignedEvent = serde_json::from_str(&args.event)?;
    let event = event.sign(&keys)?;

    if args.broadcast {
        let (_k, nostr) = config.nostr_random_client().await?;
        let event_id = nostr.send_event(event).await?;
        println!("Broadcast event {event_id}");
    } else {
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}

pub(crate) fn op_return(
    name: &str,
    pubkey: &XOnlyPublicKey,
    kind: NomenKind,
) -> anyhow::Result<()> {
    let fingerprint = Hash160::default()
        .chain_update(name.as_bytes())
        .fingerprint();
    let nsid = NsidBuilder::new(name, pubkey).finalize();
    let data = super::op_return(fingerprint, nsid, kind);

    println!("{}", hex::encode(data));

    Ok(())
}
