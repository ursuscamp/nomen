use bitcoin::hashes::hex::ToHex;
use nostr_sdk::Event;
use rusqlite::params;
use tokio_rusqlite::Connection;

use crate::{
    config::{Cli, Config},
    name::Namespace,
    util::Nsid,
};

static MIGRATIONS: [&'static str; 9] = [
    "CREATE TABLE blockchain (nsid PRIMARY KEY, blockhash, txid, vout, height);",
    "CREATE INDEX blockchain_height_dx ON blockchain(height);",
    "CREATE TABLE name_nsid (name PRIMARY KEY, nsid, root, pubkey);",
    "CREATE INDEX name_nsid_nsid_idx ON name_nsid(nsid);",
    "CREATE TABLE create_events (nsid PRIMARY KEY, pubkey, created_at, event_id, name, children);",
    "CREATE TABLE records_events (nsid, pubkey, created_at, event_id, name, records);",
    "CREATE UNIQUE INDEX records_events_unique_idx ON records_events(nsid, pubkey)",
    "CREATE INDEX records_events_created_at_idx ON records_events(created_at);",
    "CREATE VIEW name_records_vw AS
        SELECT re.name, re.records FROM blockchain b
        JOIN name_nsid nn ON b.nsid = nn.root
        JOIN create_events ce ON b.nsid = ce.nsid
        JOIN records_events re on nn.nsid = re.nsid AND nn.pubkey = re.pubkey;",
];

pub async fn initialize(config: &Config) -> anyhow::Result<()> {
    let conn = config.sqlite().await?;

    conn.call(|conn| -> anyhow::Result<()> {
        conn.execute("CREATE TABLE IF NOT EXISTS schema (version);", params![]);
        let item: usize = conn.query_row(
            "SELECT COALESCE(MAX(version) + 1, 0) FROM schema",
            params![],
            |row| row.get(0),
        )?;

        for (idx, migration) in MIGRATIONS[item..].into_iter().enumerate() {
            log::debug!("Migrating schema version {idx}");
            conn.execute(migration, params![]);
            conn.execute("INSERT INTO schema (version) VALUES (?)", params![idx]);
        }

        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn insert_namespace(
    conn: &Connection,
    nsid: String,
    blockhash: String,
    txid: String,
    vout: usize,
    height: usize,
) -> anyhow::Result<()> {
    conn.call(move |conn| -> anyhow::Result<()> {
        conn.execute(
            "INSERT INTO blockchain (nsid, blockhash, txid, vout, height) VALUES (?, ?, ?, ?, ?)",
            params![nsid, blockhash, txid, vout, height],
        );
        Ok(())
    })
    .await?;

    Ok(())
}

pub async fn next_index_height(conn: &Connection) -> anyhow::Result<usize> {
    Ok(conn
        .call(|conn| -> anyhow::Result<usize> {
            let height: usize = conn.query_row(
                "SELECT COALESCE(MAX(height), 0) + 1 FROM blockchain;",
                [],
                |row| row.get(0),
            )?;
            Ok(height)
        })
        .await?)
}

pub async fn last_create_event_time(conn: &Connection) -> anyhow::Result<u64> {
    conn.call(|conn| {
        let created_at = conn.query_row(
            "SELECT COALESCE(MAX(created_at), 0) from create_events;",
            [],
            |row| row.get(0),
        )?;
        Ok(created_at)
    })
    .await
}

pub async fn insert_create_event(
    conn: &Connection,
    event: Event,
    ns: Namespace,
) -> anyhow::Result<()> {
    conn.call(move |conn| {
        conn.execute(
            "INSERT INTO create_events (nsid, pubkey, created_at, event_id, name, children)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(nsid) DO UPDATE SET
            created_at = excluded.created_at, event_id = excluded.event_id;",
            params![
                ns.namespace_id().to_hex(),
                event.pubkey.to_hex(),
                event.created_at.as_u64(),
                event.id.to_hex(),
                &ns.0,
                serde_json::to_string(&ns.2)?
            ],
        )?;
        Ok(())
    })
    .await
}

pub async fn index_name_nsid(
    conn: &Connection,
    nsid: String,
    name: String,
    root: String,
    pubkey: String,
) -> anyhow::Result<()> {
    conn.call(move |conn| {
        // ON CONFLICT DO NOTHING ensure that if someone uploads a conflicting name,
        // we just wont index it if it already exists
        conn.execute(
            "INSERT INTO name_nsid (name, nsid, root, pubkey) VALUES (?, ?, ?, ?)
            ON CONFLICT DO NOTHING",
            params![name, nsid, root, pubkey],
        )?;
        Ok(())
    })
    .await
}

pub async fn last_records_time(conn: &Connection) -> anyhow::Result<u64> {
    conn.call(|conn| {
        Ok(conn.query_row(
            "SELECT COALESCE(MAX(created_at), 0) FROM records_events;",
            [],
            |row| row.get(0),
        )?)
    })
    .await
}

// pub async fn valid_name_nsid(
//     conn: &Connection,
//     name: String,
//     nsid: String,
// ) -> anyhow::Result<bool> {
//     conn.call(move |conn| {
//         let count: u64 = conn.query_row(
//             "SELECT COUNT(*) FROM name_nsid WHERE name = ? AND nsid = ?;",
//             params![name, nsid],
//             |row| row.get(0),
//         )?;
//         Ok(count > 0)
//     })
//     .await
// }

pub async fn insert_records_event(
    conn: &Connection,
    nsid: String,
    pubkey: String,
    created_at: u64,
    event_id: String,
    name: String,
    records: String,
) -> anyhow::Result<()> {
    conn.call(move |conn| {
        conn.execute(
            "INSERT INTO records_events (nsid, pubkey, created_at, event_id, name, records)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT (nsid, pubkey) DO UPDATE SET
            created_at = excluded.created_at,
            event_id = excluded.event_id,
            records = excluded.records;",
            params![nsid, pubkey, created_at, event_id, name, records],
        )?;
        Ok(())
    })
    .await
}

//// OLD AND BUSTED

pub async fn namespace_exists(conn: &Connection, nsid: String) -> anyhow::Result<bool> {
    Ok(conn
        .call(move |conn| -> anyhow::Result<bool> {
            let count: usize = conn.query_row(
                "SELECT COUNT(*) FROM blockchain WHERE nsid = ?;",
                params![nsid],
                |row| row.get(0),
            )?;

            Ok(count > 0)
        })
        .await?)
}

pub async fn discovered_nsids(conn: &Connection) -> anyhow::Result<Vec<String>> {
    let conn = conn.clone();
    Ok(conn
        .call(move |conn| -> anyhow::Result<Vec<String>> {
            let mut stmt =
                conn.prepare("SELECT nsid FROM namespaces WHERE status = 'discovered';")?;
            let results = stmt
                .query_map([], |row| row.get(0))?
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<String>>();
            Ok(results)
        })
        .await?)
}

pub async fn nsid_for_name(conn: &Connection, name: String) -> anyhow::Result<Option<String>> {
    Ok(conn
        .call(move |conn| {
            conn.query_row(
                "SELECT nsid FROM name_nsid WHERE name = ?;",
                params![name],
                |row| row.get(0),
            )
        })
        .await?)
}

pub async fn update_from_relay(conn: &Connection, ns: &Namespace) -> anyhow::Result<()> {
    let children = serde_json::to_string(&ns.2)?;
    let pubkey = ns.1.as_ref().to_hex();
    let name = ns.0.clone();
    let nsid = ns.namespace_id().to_string();
    conn.call(move |conn| -> anyhow::Result<()> {
        conn.execute(
            "UPDATE namespaces SET name = ?, pubkey = ?, status = \'indexed\', children = ? WHERE nsid = ?;",
            params![name, pubkey, children, nsid],
        )?;
        Ok(())
    })
    .await
}

pub async fn index_name(conn: &Connection, name: String, nsid: String) -> anyhow::Result<()> {
    conn.call(move |conn| -> anyhow::Result<()> {
        conn.execute(
            "INSERT INTO names_nsid (name, nsid) VALUES (?, ?);",
            params![name, nsid],
        )?;
        Ok(())
    })
    .await
}

pub async fn pubkey_for_nsid(conn: &Connection, nsid: String) -> anyhow::Result<Option<String>> {
    conn.call(move |conn| -> anyhow::Result<Option<String>> {
        Ok(conn.query_row(
            "SELECT pubkey FROM namespaces WHERE nsid = ?",
            params![nsid],
            |row| row.get(0),
        )?)
    })
    .await
}

pub async fn insert_records(
    conn: &Connection,
    name: String,
    created_at: u64,
    records: String,
) -> anyhow::Result<()> {
    conn.call(move |conn| {
        conn.execute(
            "INSERT INTO records (name, created_at, records) VALUES (?, ?, ?);",
            params![name, created_at, records],
        )?;
        Ok(())
    })
    .await
}
