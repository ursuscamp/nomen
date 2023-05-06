use std::sync::atomic::{AtomicBool, Ordering};

use bitcoin::{hashes::hex::ToHex, BlockHash, Txid};
use bitcoincore_rpc::RpcApi;
use sqlx::SqlitePool;

use crate::{
    config::{Cli, Config},
    db,
    util::{NomenKind, NomenTx, Nsid},
};

pub async fn index(config: &Config, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), anyhow::Error> {
    let client = config.rpc_client()?;
    let index_height = db::next_index_height(pool)
        .await?
        .max(config.starting_block_height());
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    log::info!("Starting blockchain index at height {index_height}");

    let thread = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut blockhash = client.get_block_hash(index_height as u64)?;
        let mut blockinfo = client.get_block_header_info(&blockhash)?;

        while let Some(next_hash) = blockinfo.next_block_hash {
            // If the channel is closed, let's stop
            if sender.is_closed() {
                log::info!("Stopping index operation.");
                break;
            }

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
                                    sender.blocking_send((
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

        Ok(())
    });

    let guard = elegant_departure::get_shutdown_guard();
    'select: loop {
        tokio::select! {
            msg = receiver.recv() => {
                match msg {
                    Some((fingerprint, nsid, blockhash, txid, blocktime, blockheight, txheight, vout, kind)) => {
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
                    None => break 'select,
                }
            }
            _ = guard.wait() => {
                receiver.close();
                // std::mem::drop(receiver);
                break 'select;
            }
        }
    }

    // while let Some(
    //     ((fingerprint, nsid, blockhash, txid, blocktime, blockheight, txheight, vout, kind)),
    // ) = receiver.recv().await
    // {
    //     if let Err(e) = index_output(
    //         pool,
    //         fingerprint,
    //         nsid,
    //         &blockhash,
    //         &txid,
    //         blocktime,
    //         blockheight,
    //         txheight,
    //         vout,
    //         kind,
    //     )
    //     .await
    //     {
    //         log::error!("Index error: {e}");
    //     }
    // }

    // let guard = elegant_departure::get_shutdown_guard();
    // let mut indexed_txs = tokio::select! {
    //     _ = guard.wait() => {
    //         log::info!("Index shutdown requested.");
    //         vec![]
    //     },
    //     Ok(Ok(v)) = thread => v,
    // };

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
