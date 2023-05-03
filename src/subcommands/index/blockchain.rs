use bitcoin::{hashes::hex::ToHex, BlockHash, Txid};
use bitcoincore_rpc::RpcApi;
use sqlx::SqlitePool;

use crate::{
    config::Config,
    db,
    util::{NomenKind, NomenTx, Nsid},
};

pub async fn index(config: &Config, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), anyhow::Error> {
    let client = config.rpc_client()?;
    let index_height = db::next_index_height(pool)
        .await?
        .max(config.starting_block_height());

    log::info!("Starting blockchain index at height {index_height}");

    let indexed_txs = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut index_txs = Vec::new();
        let mut blockhash = client.get_block_hash(index_height as u64)?;
        let mut blockinfo = client.get_block_header_info(&blockhash)?;

        while let Some(next_hash) = blockinfo.next_block_hash {
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

            let block = client.get_block(&blockhash)?;

            for (txheight, tx) in block.txdata.iter().enumerate() {
                for (vout, output) in tx.output.iter().enumerate() {
                    if output.script_pubkey.is_op_return() {
                        let b = &output.script_pubkey.as_bytes()[2..];

                        // Pre-check if it starts with NOM, so we can filter out some unnecessary errors from the logs
                        if b.starts_with(b"NOM") {
                            match NomenTx::try_from(b) {
                                Ok(NomenTx {
                                    fingerprint,
                                    nsid,
                                    kind,
                                }) => {
                                    index_txs.push((
                                        fingerprint,
                                        nsid,
                                        blockhash,
                                        tx.txid(),
                                        blockinfo.time,
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
            blockinfo = client.get_block_header_info(&blockhash)?;
        }

        Ok(index_txs)
    })
    .await??;

    for (fingerprint, nsid, blockhash, txid, blocktime, blockheight, txheight, vout, kind) in
        indexed_txs
    {
        if let Err(e) = index_output(
            pool,
            fingerprint,
            nsid,
            &blockhash,
            &txid,
            blocktime,
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

#[allow(clippy::too_many_arguments)]
async fn index_output(
    conn: &SqlitePool,
    fingerprint: [u8; 5],
    nsid: Nsid,
    blockhash: &BlockHash,
    txid: &Txid,
    blocktime: usize,
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
        fingerprint,
        nsid,
        blockhash.to_hex(),
        txid.to_hex(),
        blocktime,
        blockheight,
        txheight,
        vout,
        kind,
    )
    .await?;
    Ok(())
}
