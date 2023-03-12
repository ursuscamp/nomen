use anyhow::anyhow;
use bitcoin::{hashes::hex::ToHex, BlockHash, Network, Txid};
use bitcoincore_rpc::RpcApi;

use crate::{config::Config, db, name::Namespace};

pub async fn index_blockchain(
    config: &Config,
    confirmations: usize,
    height: Option<usize>,
) -> anyhow::Result<()> {
    let height = index_height(height, config).await?;
    log::info!("Starting index from block height: {height}");

    let client = config.rpc_client()?;

    let mut blockhash = client.get_block_hash(height as u64)?;
    let mut blockinfo = client.get_block_info(&blockhash)?;
    while let Some(next_hash) = blockinfo.nextblockhash {
        if (blockinfo.confirmations as usize) < confirmations {
            log::info!(
                "Minimum confirmations not met at block height {}.",
                blockinfo.height
            );
            break; // Minimum confirmations required.
        }
        log_height(blockinfo.height as u64);

        for txid in blockinfo.tx {
            let tx = client.get_raw_transaction(&txid, None)?;

            for (vout, output) in tx.output.into_iter().enumerate() {
                if output.script_pubkey.is_op_return() {
                    let b = &output.script_pubkey.as_bytes()[2..];
                    if b.starts_with(b"ind") {
                        let b = &b[3..];
                        match parse_ind_output(&b) {
                            Ok(b) => {
                                match index_output(b, &blockhash, &txid, vout) {
                                    Err(err) => log::error!("Index error: {err}"),
                                    _ => {}
                                };
                            }
                            Err(e) => log::error!("Index error: {e}"),
                        }
                    }
                }
            }
        }

        blockhash = next_hash;
        blockinfo = client.get_block_info(&blockhash)?;
    }

    log::info!("Blockchain index complete.");
    Ok(())
}

async fn index_height(height: Option<usize>, config: &Config) -> Result<usize, anyhow::Error> {
    let conn = config.sqlite().await?;
    let db_height = db::next_index_height(&conn).await;
    Ok(height
        .map(Result::Ok)
        .or(Some(db_height))
        .or_else(|| Some(starting_blockheight(config.network.unwrap())))
        .expect("starting height")?)
}

fn index_output(
    bytes: Vec<u8>,
    blockhash: &BlockHash,
    txid: &Txid,
    vout: usize,
) -> anyhow::Result<()> {
    let nsid = bytes.to_hex();
    log::info!("IND output found: {}", nsid);
    if bytes.len() != 20 {
        return Err(anyhow::anyhow!("Unexpected IND length"));
    }

    let nstree = db::namespaces()?;
    if nstree.contains_key(&bytes)? {
        log::info!("NSID {nsid} already index, skipping");
        return Ok(());
    }

    let namespace = db::NamespaceModel::new_detected(&nsid, None, blockhash, txid, vout)?;
    nstree.insert(&bytes, namespace.encode()?)?;
    Ok(())
}

fn log_height(height: u64) {
    if height % 10 == 0 {
        log::info!("Indexing block height {height}");
    } else {
        log::debug!("Indexing block height {height}");
    }
}

fn starting_blockheight(network: Network) -> anyhow::Result<usize> {
    match network {
        Network::Bitcoin => Err(anyhow!("Unsupported network {}", network)),
        Network::Testnet => Err(anyhow!("Unsupported network {}", network)),
        Network::Signet => Err(anyhow!("Unsupported network {}", network)),
        Network::Regtest => Ok(1),
    }
}

fn parse_ind_output(byte: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut b = byte.into_iter();
    let (ind_ver, ind_type) = (b.next(), b.next());
    match (ind_ver, ind_type) {
        (Some(&0), Some(&0)) => Ok(b.copied().collect()),
        _ => Err(anyhow!("Invalid ind code")),
    }
}
