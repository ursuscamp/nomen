use anyhow::anyhow;
use bitcoin::{hashes::hex::ToHex, BlockHash, Txid};
use bitcoincore_rpc::RpcApi;
use itertools::Itertools;
use nostr_sdk::{Event, Filter};
use sqlx::SqlitePool;

use crate::{
    config::Config,
    db,
    util::{NameKind, NomenKind, NomenTx, Nsid},
};

pub async fn index(config: &Config, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), anyhow::Error> {
    let client = config.rpc_client()?;
    let index_height = db::next_index_height(pool).await?;

    log::info!("Starting blockchain index at height {index_height}");

    let indexed_txs = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut index_txs = Vec::new();
        let mut blockhash = client.get_block_hash(index_height as u64)?;
        let mut blockinfo = client.get_block_info(&blockhash)?;

        while let Some(next_hash) = blockinfo.nextblockhash {
            if (blockinfo.confirmations as usize) < 3 {
                log::info!(
                    "Minimum confirmations not met at block height {}.",
                    blockinfo.height
                );
                break;
            }
            if blockinfo.height % 10 == 0 {
                log::info!("Index block height {}", blockinfo.height);
            }

            for (txheight, txid) in blockinfo.tx.iter().enumerate() {
                let tx = client.get_raw_transaction(txid, None)?;
                for (vout, output) in tx.output.into_iter().enumerate() {
                    if output.script_pubkey.is_op_return() {
                        let b = &output.script_pubkey.as_bytes()[2..];

                        // Pre-check if it starts with NOM, so we can filter out some unnecessary errors from the logs
                        if b.starts_with(b"NOM") {
                            match NomenTx::try_from(b) {
                                Ok(NomenTx { nsid, kind }) => {
                                    index_txs.push((
                                        nsid,
                                        blockhash,
                                        *txid,
                                        blockinfo.height,
                                        txheight,
                                        vout,
                                        kind,
                                    ));
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

        Ok(index_txs)
    })
    .await??;

    for (nsid, blockhash, txid, blockheight, txheight, vout, kind) in indexed_txs {
        if let Err(e) = index_output(
            pool,
            nsid,
            &blockhash,
            &txid,
            blockheight,
            txheight,
            vout,
            kind,
        )
        .await
        {
            log::error!("Index error: {e}");
        }
    }

    log::info!("Blockchain index complete.");
    Ok(())
}

fn parse_ind_output(byte: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut b = byte.iter();
    let (ind_ver, ind_type) = (b.next(), b.next());
    match (ind_ver, ind_type) {
        (Some(&0), Some(&0)) => Ok(b.copied().collect()),
        _ => Err(anyhow!("Invalid ind code")),
    }
}

#[allow(clippy::too_many_arguments)]
async fn index_output(
    conn: &SqlitePool,
    nsid: Nsid,
    blockhash: &BlockHash,
    txid: &Txid,
    blockheight: usize,
    txheight: usize,
    vout: usize,
    kind: NomenKind,
) -> anyhow::Result<()> {
    log::info!("NOM output found: {}", nsid);
    if nsid.len() != 20 {
        return Err(anyhow::anyhow!("Unexpected NOM length"));
    }

    db::insert_blockchain(
        conn,
        nsid,
        blockhash.to_hex(),
        txid.to_hex(),
        blockheight,
        txheight,
        vout,
        kind,
    )
    .await?;
    Ok(())
}
