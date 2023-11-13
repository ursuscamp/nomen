#![allow(clippy::module_name_repetitions)]

use nomen_core::{Hash160, Name, Nsid};
use nostr_sdk::EventId;
use secp256k1::XOnlyPublicKey;
use sqlx::{FromRow, Sqlite, SqlitePool};

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
    pub v1_upgrade_blockheight: Option<i64>,
    pub v1_upgrade_txid: Option<String>,
}

pub async fn details(conn: &SqlitePool, query: &str) -> anyhow::Result<NameDetails> {
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

pub async fn records(conn: &SqlitePool, name: String) -> anyhow::Result<Option<NameRecords>> {
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

#[allow(clippy::too_many_arguments)]
pub async fn insert_name_event(
    conn: &SqlitePool,
    name: Name,
    fingerprint: [u8; 5],
    nsid: Nsid,
    pubkey: XOnlyPublicKey,
    created_at: i64,
    event_id: EventId,
    records: String,
    raw_event: String,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_name_event.sql"))
        .bind(name.to_string())
        .bind(hex::encode(fingerprint))
        .bind(nsid.to_string())
        .bind(pubkey.to_string())
        .bind(created_at)
        .bind(event_id.to_string())
        .bind(records)
        .bind(raw_event)
        .execute(conn)
        .await?;
    Ok(())
}

pub type NameAndKey = (String, String);

pub async fn fetch_all(conn: &SqlitePool) -> anyhow::Result<Vec<NameAndKey>> {
    let rows = sqlx::query_as::<_, NameAndKey>("SELECT name, pubkey FROM valid_names_vw;")
        .fetch_all(conn)
        .await?;
    Ok(rows)
}

pub async fn check_availability(
    conn: impl sqlx::Executor<'_, Database = Sqlite> + Copy,
    name: &str,
) -> anyhow::Result<bool> {
    let fp = hex::encode(
        Hash160::default()
            .chain_update(name.as_bytes())
            .fingerprint(),
    );
    let (a,) = sqlx::query_as::<_, (bool,)>(
        "SELECT COUNT(*) = 0 FROM valid_names_vw WHERE fingerprint = ?;",
    )
    .bind(&fp)
    .fetch_one(conn)
    .await?;
    Ok(a)
}

pub async fn last_records_time(conn: &SqlitePool) -> anyhow::Result<u64> {
    let (t,) = sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(created_at), 0) FROM name_events;")
        .fetch_one(conn)
        .await?;
    Ok(t as u64)
}
