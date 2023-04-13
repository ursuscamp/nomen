use std::collections::HashMap;

use bitcoin::{hashes::hex::ToHex, XOnlyPublicKey};
use nostr_sdk::EventId;
use sqlx::{FromRow, SqlitePool};

use crate::{
    config::Config,
    util::{NomenKind, Nsid},
};

static MIGRATIONS: [&str; 10] = [
    "CREATE TABLE blockchain (id INTEGER PRIMARY KEY, nsid, blockhash, txid, blockheight, txheight, vout, kind);",
    "CREATE TABLE name_events (nsid, name, pubkey, created_at, event_id, content);",
    "CREATE UNIQUE INDEX name_events_unique_idx ON name_events(nsid)",
    "CREATE TABLE records_events (name, pubkey, created_at, event_id, records);",
    "CREATE UNIQUE INDEX records_events_unique_idx ON records_events(name, pubkey)",
    "CREATE INDEX records_events_created_at_idx ON records_events(created_at);",
    "CREATE VIEW ordered_blockchain_vw AS
        SELECT b.* FROM blockchain b
        ORDER BY b.blockheight, b.txheight, b.vout",
    "CREATE VIEW name_vw AS
        SELECT ne.* FROM ordered_blockchain_vw b
        JOIN name_events ne on b.nsid = ne.nsid;",
    "CREATE VIEW records_vw AS
        SELECT nvw.nsid, nvw.name, re.records FROM name_vw nvw
        LEFT JOIN records_events re ON nvw.name = re.name AND nvw.pubkey = re.pubkey;",
    "CREATE VIEW detail_vw AS
        SELECT b.nsid, b.blockhash, b.txid, b.vout, b.blockheight, ne.name, COALESCE(re.records, '{}') as records
        FROM ordered_blockchain_vw b
        JOIN name_events ne on b.nsid = ne.nsid
        LEFT JOIN records_events re on ne.name = re.name AND ne.pubkey = re.pubkey;"
];

pub async fn initialize(config: &Config) -> anyhow::Result<SqlitePool> {
    let conn = config.sqlite().await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS schema (version);")
        .execute(&conn)
        .await?;

    let (version,) =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(version) + 1, 0) FROM schema")
            .fetch_one(&conn)
            .await?;

    for (idx, migration) in MIGRATIONS[version as usize..].iter().enumerate() {
        log::debug!("Migrations schema version {idx}");
        sqlx::query(migration).execute(&conn).await?;
        sqlx::query("INSERT INTO schema (version) VALUES (?);")
            .bind(idx as i64)
            .execute(&conn)
            .await?;
    }

    Ok(conn)
}

// TODO: combine these arguments into a simpler set for <8
#[allow(clippy::too_many_arguments)]
pub async fn insert_blockchain(
    conn: &SqlitePool,
    nsid: Nsid,
    blockhash: String,
    txid: String,
    blockheight: usize,
    txheight: usize,
    vout: usize,
    kind: NomenKind,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_namespace.sql"))
        .bind(nsid.to_hex())
        .bind(blockhash)
        .bind(txid)
        .bind(blockheight as i64)
        .bind(txheight as i64)
        .bind(vout as i64)
        .bind(kind.to_string())
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn next_index_height(conn: &SqlitePool) -> anyhow::Result<usize> {
    let (h,) =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(blockheight), 0) + 1 FROM blockchain;")
            .fetch_one(conn)
            .await?;
    Ok(h as usize)
}

pub async fn last_create_event_time(conn: &SqlitePool) -> anyhow::Result<u64> {
    let (t,) = sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(created_at), 0) from name_events;")
        .fetch_one(conn)
        .await?;
    Ok(t as u64)
}

pub async fn insert_create_event(
    conn: &SqlitePool,
    nsid: Nsid,
    pubkey: XOnlyPublicKey,
    created_at: i64,
    event_id: EventId,
    name: String,
    children: String,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_name_event.sql"))
        .bind(nsid.to_hex())
        .bind(pubkey.to_hex())
        .bind(created_at)
        .bind(event_id.to_hex())
        .bind(name)
        .bind(children)
        .execute(conn)
        .await?;

    Ok(())
}

#[derive(FromRow)]
pub struct NameDetails {
    pub blockhash: String,
    pub txid: String,
    pub vout: i64,
    pub blockheight: i64,
    pub name: String,
    pub records: String,
}

pub async fn name_details(conn: &SqlitePool, nsid: Nsid) -> anyhow::Result<NameDetails> {
    let details = sqlx::query_as::<_, NameDetails>("SELECT * FROM detail_vw WHERE nsid = ?")
        .bind(nsid.to_hex())
        .fetch_one(conn)
        .await?;
    Ok(details)
}

pub async fn last_records_time(conn: &SqlitePool) -> anyhow::Result<u64> {
    let (t,) =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(created_at), 0) FROM records_events;")
            .fetch_one(conn)
            .await?;
    Ok(t as u64)
}

pub async fn insert_records_event(
    conn: &SqlitePool,
    pubkey: XOnlyPublicKey,
    created_at: i64,
    event_id: EventId,
    name: String,
    records: String,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_records_event.sql"))
        .bind(pubkey.to_string())
        .bind(created_at)
        .bind(event_id.to_string())
        .bind(name)
        .bind(records)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn name_records(
    conn: &SqlitePool,
    name: String,
) -> anyhow::Result<Option<HashMap<String, String>>> {
    let content =
        sqlx::query_as::<_, (String,)>("SELECT records from name_records_vw where name = ?;")
            .bind(name)
            .fetch_optional(conn)
            .await?;

    let records = content
        .map(|s| s.0)
        .map(|records| serde_json::from_str(&records))
        .transpose()?;
    Ok(records)
}

pub async fn top_level_names(conn: &SqlitePool) -> anyhow::Result<Vec<(String, String)>> {
    Ok(
        sqlx::query_as::<_, (String, String)>("SELECT * FROM name_vw;")
            .fetch_all(conn)
            .await?,
    )
}
