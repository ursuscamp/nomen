#![allow(clippy::module_name_repetitions)]

use nomen_core::Hash160;
use sqlx::{FromRow, SqlitePool};

#[derive(FromRow)]
pub struct NameDetails {
    pub blockhash: String,
    pub txid: String,
    pub blocktime: i64,
    pub vout: i64,
    pub blockheight: i64,
    pub name: String,
    pub records: String,
    pub pubkey: String,
    pub protocol: i64,
}

pub async fn name_details(conn: &SqlitePool, query: &str) -> anyhow::Result<NameDetails> {
    let details = sqlx::query_as::<_, NameDetails>(
        "SELECT * from valid_names_records_vw vn WHERE vn.nsid = ? or vn.name = ?",
    )
    .bind(query)
    .bind(query)
    .fetch_one(conn)
    .await?;
    Ok(details)
}

#[derive(FromRow)]
pub struct NameRecords {
    pub blockhash: String,
    pub txid: String,
    pub fingerprint: String,
    pub nsid: String,
    pub protocol: i64,
    pub records: String,
}

pub async fn name_records(conn: &SqlitePool, name: String) -> anyhow::Result<Option<NameRecords>> {
    let fingerprint = Hash160::default()
        .chain_update(name.as_bytes())
        .fingerprint();
    let records = sqlx::query_as::<_, NameRecords>(
        "SELECT vn.blockhash, vn.txid, vn.fingerprint, vn.nsid, vn.protocol, coalesce(ne.records, '{}') as records
        FROM valid_names_vw vn
        JOIN name_events ne ON vn.nsid = ne.nsid
        WHERE vn.fingerprint = ? LIMIT 1;",
    )
    .bind(hex::encode(fingerprint))
    .fetch_optional(conn)
    .await?;
    Ok(records)
}

pub async fn top_level_names(
    conn: &SqlitePool,
    query: Option<String>,
) -> anyhow::Result<Vec<(String, String)>> {
    let sql = match query {
        Some(q) => sqlx::query_as::<_, (String, String)>(
            "SELECT nsid, name FROM valid_names_vw WHERE name IS NOT NULL AND instr(name, ?) ORDER BY name;",
        )
        .bind(q.to_lowercase()),
        None => sqlx::query_as::<_, (String, String)>(
            "SELECT nsid, name FROM valid_names_vw WHERE name IS NOT NULL ORDER BY name;",
        ),
    };

    Ok(sql.fetch_all(conn).await?)
}
