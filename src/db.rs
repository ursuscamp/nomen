use bitcoin::hashes::hex::ToHex;
use rusqlite::params;
use tokio_rusqlite::Connection;

use crate::{config::Config, name::Namespace, util::Nsid};

static MIGRATIONS: [&'static str; 5] = [
    "CREATE TABLE namespaces (nsid PRIMARY KEY, name, pubkey, blockhash, txid, vout, height, status, children);",
    "CREATE INDEX namespaces_status_idx ON namespaces(status);",
    "CREATE TABLE names_nsid (name PRIMARY KEY, nsid);",
    "CREATE TABLE records (name PRIMARY KEY, created_at, records);",
    "CREATE INDEX records_created_at_idx ON records(created_at);"
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

pub async fn discover_namespace(
    conn: &Connection,
    nsid: String,
    blockhash: String,
    txid: String,
    vout: usize,
    height: usize,
) -> anyhow::Result<()> {
    conn.call(move |conn| -> anyhow::Result<()> {
        conn.execute(
            "INSERT INTO namespaces (nsid, blockhash, txid, vout, height, status) VALUES (?, ?, ?, ?, ?, 'discovered')",
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
                "SELECT COALESCE(MAX(height), 0) + 1 FROM namespaces;",
                [],
                |row| row.get(0),
            )?;
            Ok(height)
        })
        .await?)
}

pub async fn namespace_exists(conn: &Connection, nsid: String) -> anyhow::Result<bool> {
    Ok(conn
        .call(move |conn| -> anyhow::Result<bool> {
            let count: usize = conn.query_row(
                "SELECT COUNT(*) FROM namespaces WHERE nsid = ?;",
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
                "SELECT nsid FROM names_nsid WHERE name = ?;",
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
