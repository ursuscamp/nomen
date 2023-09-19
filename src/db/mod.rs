use std::collections::HashMap;

use bitcoin::{BlockHash, Txid};
use nostr_sdk::EventId;
use secp256k1::XOnlyPublicKey;
use sqlx::{FromRow, SqlitePool};

use crate::{
    config::{Cli, Config},
    util::{self, Hash160, Name, NomenKind, Nsid},
};

static MIGRATIONS: [&str; 27] = [
    "CREATE TABLE index_height (blockheight INTEGER PRIMARY KEY, blockhash);",
    "CREATE TABLE blockchain (id INTEGER PRIMARY KEY, fingerprint, nsid, blockhash, txid, blocktime, blockheight, txheight, vout, kind, indexed_at);",
    "CREATE TABLE name_events (name, fingerprint, nsid, pubkey, created_at, event_id, records, indexed_at, raw_event);",
    "CREATE UNIQUE INDEX name_events_unique_idx ON name_events(name, pubkey);",
    "CREATE INDEX name_events_created_at_idx ON name_events(created_at);",
    "CREATE TABLE transfer_events (nsid, name, fingerprint, pubkey, created_at, event_id, content, indexed_at, raw_event);",
    "CREATE UNIQUE INDEX transfer_events_unique_idx ON transfer_events(nsid)",

    // We order by blockheight -> txheight (height of tx inside block) and then vout (output inside tx)
    // to make sure we are always looking in exact blockchain order
    "CREATE VIEW ordered_blockchain_vw AS
        SELECT b.* FROM blockchain b
        ORDER BY b.blockheight, b.txheight, b.vout",

    // Someone could theoretically try to claim a name a second time, we want to rank each blockchain event
    // in order, partitioned by name. So if Person A claims 'domain-name' first, then Person B also claims 'domain-name'
    // second, then Person A will be ranked 1, and Person B will be ranked 2.
    "CREATE VIEW ranked_name_vw AS
        SELECT ne.*, ROW_NUMBER() OVER (PARTITION BY ne.name) as row
        FROM ordered_blockchain_vw b
        JOIN name_events ne on b.fingerprint = ne.fingerprint AND b.nsid = ne.nsid
        WHERE b.kind = 'create';",

    // We select everyone that has rank 1. This is always going to be first claimed on blockchain.
    "CREATE VIEW name_vw AS 
        SELECT * FROM ranked_name_vw WHERE row = 1;",

    // Starting with a valid name event, follow the graph recursively to each successive transfer_event (if such exists),
    // connecting pubkey -> content (next pubkey) -> pubkey -> content (next pubkey), etc. The resulting query returns
    // the successive owners of each name
    "CREATE VIEW ownership_chain_vw AS
        WITH RECURSIVE owners(name, pk) as (
            SELECT name, pubkey FROM name_vw
            UNION ALL
            SELECT te.name, te.content
                FROM transfer_events te
                JOIN owners ON te.pubkey = owners.pk AND te.name = owners.name
                JOIN blockchain b on te.nsid = b.nsid AND te.fingerprint = b.fingerprint
                WHERE b.kind = 'transfer'
        )
        SELECT name, pk FROM owners;",

    // Partition over the names, and only return the final value (the latest owner)
    "CREATE VIEW owners_vw AS
        SELECT DISTINCT name, last_value(pubkey) OVER (PARTITION BY name) AS pubkey 
        FROM ownership_chain_vw;",

    // This table is used to cache the owners_vw results, to avoid a full graph traversal every time.
    "CREATE TABLE name_owners (name, pubkey);",

    "CREATE VIEW records_vw AS
        SELECT ne.* FROM name_owners no
        JOIN name_events ne on no.name = ne.name AND no.pubkey = ne.pubkey
        ORDER BY ne.created_at DESC;",

    "CREATE VIEW detail_vw AS
        SELECT 
            b.nsid,
            b.blockhash,
            b.blocktime,
            b.txid,
            b.vout,
            b.blockheight,
            r.name, 
            COALESCE(r.records, '{}') as records,
            r.pubkey,
            r.created_at as records_created_at
        FROM records_vw r
        JOIN ordered_blockchain_vw b ON r.fingerprint = b.fingerprint AND r.nsid = b.nsid;",

    "CREATE TABLE event_log (created_at, type, data);",

    // This represents claims put on the blockchain that don't have any associated Nostr events.
    "CREATE VIEW uncorroborated_claims_vw AS
        SELECT b.* FROM ordered_blockchain_vw b
        LEFT JOIN name_events ne on b.fingerprint = ne.fingerprint AND b.nsid = ne.nsid
        WHERE b.kind = 'create' AND ne.nsid IS NULL;",

    // The following changes came from https://github.com/ursuscamp/nomen/issues/9 in which a name was
    // being indexed twice. It turns out that the same NSID was sent across multiple transactions. This
    // was unexepected behavior by the indexer.
    "DROP VIEW ordered_blockchain_vw;",

    "CREATE VIEW ordered_blockchain_vw AS
        SELECT b.*, row_number() OVER (PARTITION BY b.nsid) as row
        FROM blockchain b
        ORDER BY b.blockheight, b.txheight, b.vout;",

    "DROP VIEW ranked_name_vw;",

    "CREATE VIEW ranked_name_vw AS
        SELECT ne.*, ROW_NUMBER() OVER (PARTITION BY ne.name) as row
        FROM ordered_blockchain_vw b
        JOIN name_events ne on b.fingerprint = ne.fingerprint AND b.nsid = ne.nsid
        WHERE b.kind = 'create' and b.row = 1;",

    "DROP VIEW detail_vw",
        
    "CREATE VIEW detail_vw AS
    SELECT 
        b.nsid,
        b.blockhash,
        b.blocktime,
        b.txid,
        b.vout,
        b.blockheight,
        r.name, 
        COALESCE(r.records, '{}') as records,
        r.pubkey,
        r.created_at as records_created_at
    FROM records_vw r
    JOIN ordered_blockchain_vw b ON r.fingerprint = b.fingerprint AND r.nsid = b.nsid
    WHERE b.row = 1;",
    // END issue 9 changes

    "DROP VIEW ownership_chain_vw;",

    "DROP VIEW owners_vw;",

    "CREATE VIEW owners_vw AS
        SELECT name, pubkey FROM name_vw;",

    "DROP TABLE transfer_events;",
];

pub async fn initialize(config: &Config) -> anyhow::Result<SqlitePool> {
    let conn = config.sqlite().await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS schema (version);")
        .execute(&conn)
        .await?;

    let (version,) =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(version) + 1, 0) FROM schema;")
            .fetch_one(&conn)
            .await?;

    for (idx, migration) in MIGRATIONS[version as usize..].iter().enumerate() {
        let version = idx as i64 + version;
        let mut tx = conn.begin().await?;
        log::debug!("Migrations schema version {version}");
        sqlx::query(migration).execute(&mut tx).await?;
        sqlx::query("INSERT INTO schema (version) VALUES (?);")
            .bind(version)
            .execute(&mut tx)
            .await?;
        tx.commit().await?;
    }

    Ok(conn)
}

// TODO: combine these arguments into a simpler set for <8
#[allow(clippy::too_many_arguments)]
pub async fn insert_blockchain(
    conn: &SqlitePool,
    fingerprint: [u8; 5],
    nsid: Nsid,
    blockhash: String,
    txid: String,
    blocktime: usize,
    blockheight: usize,
    txheight: usize,
    vout: usize,
    kind: NomenKind,
) -> anyhow::Result<()> {
    sqlx::query(include_str!("./queries/insert_namespace.sql"))
        .bind(hex::encode(fingerprint))
        .bind(nsid.to_string())
        .bind(blockhash)
        .bind(txid)
        .bind(blocktime as i64)
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
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(blockheight), 0) + 1 FROM index_height;")
            .fetch_one(conn)
            .await?;
    Ok(h as usize)
}

pub async fn insert_index_height(
    conn: &SqlitePool,
    height: i64,
    blockhash: &BlockHash,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO index_height (blockheight, blockhash) VALUES (?, ?) ON CONFLICT DO NOTHING;",
    )
    .bind(height)
    .bind(blockhash.to_string())
    .execute(conn)
    .await?;
    Ok(())
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
        .bind(nsid.to_string())
        .bind(pubkey.to_string())
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
    pub blocktime: i64,
    pub vout: i64,
    pub blockheight: i64,
    pub name: String,
    pub records: String,
    pub pubkey: String,
    pub records_created_at: i64,
}

pub async fn name_details(conn: &SqlitePool, query: &str) -> anyhow::Result<NameDetails> {
    let details =
        sqlx::query_as::<_, NameDetails>("SELECT * FROM detail_vw WHERE nsid = ? or name = ?")
            .bind(query)
            .bind(query)
            .fetch_one(conn)
            .await?;
    Ok(details)
}

pub async fn last_records_time(conn: &SqlitePool) -> anyhow::Result<u64> {
    let (t,) = sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(created_at), 0) FROM name_events;")
        .fetch_one(conn)
        .await?;
    Ok(t as u64)
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

pub async fn name_records(
    conn: &SqlitePool,
    name: String,
) -> anyhow::Result<Option<HashMap<String, String>>> {
    let content =
        sqlx::query_as::<_, (String,)>("SELECT records from detail_vw where name = ? LIMIT 1;")
            .bind(name)
            .fetch_optional(conn)
            .await?;

    let records = content
        .map(|s| s.0)
        .map(|records| serde_json::from_str(&records))
        .transpose()?;
    Ok(records)
}

pub async fn top_level_names(
    conn: &SqlitePool,
    query: Option<String>,
) -> anyhow::Result<Vec<(String, String)>> {
    let sql = match query {
        Some(q) => sqlx::query_as::<_, (String, String)>(
            "SELECT nsid, name FROM detail_vw WHERE instr(name, ?) ORDER BY name;",
        )
        .bind(q.to_lowercase()),
        None => {
            sqlx::query_as::<_, (String, String)>("SELECT nsid, name FROM detail_vw ORDER BY name;")
        }
    };

    Ok(sql.fetch_all(conn).await?)
}

pub async fn save_event(conn: &SqlitePool, evt_type: &str, evt_data: &str) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO event_log (created_at, type, data) VALUES (unixepoch(), ?, ?);")
        .bind(evt_type)
        .bind(evt_data)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn last_index_time(conn: &SqlitePool) -> anyhow::Result<i64> {
    let (created_at,) = sqlx::query_as::<_, (i64,)>(
        "SELECT created_at FROM event_log WHERE type = 'index' ORDER BY created_at DESC LIMIT 1;",
    )
    .fetch_one(conn)
    .await?;

    Ok(created_at)
}

pub async fn name_available(conn: &SqlitePool, name: &str) -> anyhow::Result<bool> {
    let fingerprint = hex::encode(
        Hash160::default()
            .chain_update(name.as_bytes())
            .fingerprint(),
    );
    let (count,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM blockchain where fingerprint = ? AND kind = 'create';",
    )
    .bind(&fingerprint)
    .fetch_one(conn)
    .await?;
    Ok(count == 0)
}

pub async fn name_owner(conn: &SqlitePool, name: &str) -> anyhow::Result<Option<XOnlyPublicKey>> {
    let pubkey = sqlx::query_as::<_, (String,)>("SELECT pubkey FROM name_owners WHERE name = ?;")
        .bind(name)
        .fetch_optional(conn)
        .await?;

    Ok(pubkey.and_then(|(pk,)| pk.parse::<XOnlyPublicKey>().ok()))
}

pub async fn uncorroborated_claims(conn: &SqlitePool) -> anyhow::Result<Vec<String>> {
    Ok(
        sqlx::query_as::<_, (String,)>("SELECT txid FROM uncorroborated_claims_vw;")
            .fetch_all(conn)
            .await?
            .into_iter()
            .map(|s| s.0)
            .collect(),
    )
}

#[derive(Debug, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct UncorroboratedClaim {
    pub fingerprint: String,
    pub nsid: String,
    pub blockhash: String,
    pub txid: String,
    pub blocktime: i64,
    pub blockheight: i64,
    pub txheight: i64,
    pub vout: i64,
    pub indexed_at: i64,
}

impl UncorroboratedClaim {
    pub fn fmt_blocktime(&self) -> anyhow::Result<String> {
        util::format_time(self.blocktime)
    }

    pub fn fmt_indexed_at(&self) -> anyhow::Result<String> {
        util::format_time(self.indexed_at)
    }
}

pub async fn uncorroborated_claim(
    conn: &SqlitePool,
    txid: &str,
) -> anyhow::Result<UncorroboratedClaim> {
    Ok(sqlx::query_as(
        "SELECT fingerprint, nsid, blockhash, txid, blocktime, blockheight, txheight, vout, indexed_at
        FROM uncorroborated_claims_vw WHERE txid = ?;").bind(txid).fetch_one(conn).await?)
}

pub mod stats {
    use sqlx::SqlitePool;

    pub async fn known_names(conn: &SqlitePool) -> anyhow::Result<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>("SELECT count(*) FROM detail_vw;")
            .fetch_one(conn)
            .await?;
        Ok(count)
    }

    pub async fn index_height(conn: &SqlitePool) -> anyhow::Result<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>("SELECT max(blockheight) FROM index_height;")
            .fetch_one(conn)
            .await?;
        Ok(count)
    }

    pub async fn nostr_events(conn: &SqlitePool) -> anyhow::Result<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>(
            "
            WITH events as (
                SELECT count(*) as count FROM name_events
            )
            SELECT SUM(count) FROM events;
        ",
        )
        .fetch_one(conn)
        .await?;
        Ok(count)
    }
}
