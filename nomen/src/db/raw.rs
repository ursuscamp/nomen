use bitcoin::{BlockHash, Txid};
use sqlx::{sqlite::SqliteRow, Executor, FromRow, Row, Sqlite};
use std::str::FromStr;

pub struct RawBlockchain {
    pub blockhash: BlockHash,
    pub txid: Txid,
    pub blocktime: usize,
    pub blockheight: usize,
    pub txheight: usize,
    pub vout: usize,
    pub data: Vec<u8>,
}

impl FromRow<'_, SqliteRow> for RawBlockchain {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(RawBlockchain {
            blockhash: BlockHash::from_str(row.try_get("blockhash")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            txid: Txid::from_str(row.try_get("txid")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            blocktime: row.try_get::<i64, _>("blocktime")? as usize,
            blockheight: row.try_get::<i64, _>("blockheight")? as usize,
            txheight: row.try_get::<i64, _>("txheight")? as usize,
            vout: row.try_get::<i64, _>("vout")? as usize,
            data: hex::decode(row.try_get::<String, _>("data")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        })
    }
}

pub async fn insert_raw_blockchain(
    conn: impl Executor<'_, Database = Sqlite>,
    raw: &RawBlockchain,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_raw_blockchain.sql"))
        .bind(raw.blockhash.to_string())
        .bind(raw.txid.to_string())
        .bind(raw.blocktime as i64)
        .bind(raw.blockheight as i64)
        .bind(raw.txheight as i64)
        .bind(raw.vout as i64)
        .bind(hex::encode(&raw.data))
        .execute(conn)
        .await?;
    Ok(())
}
