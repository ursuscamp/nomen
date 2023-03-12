use rusqlite::params;
use tokio_rusqlite::Connection;

use crate::config::Config;

static MIGRATIONS: [&'static str; 2] = [
    "CREATE TABLE namespaces (nsid PRIMARY KEY, name, pubkey, blockhash, txid, vout, status, children);",
    "CREATE INDEX namespaces_status_idx ON namespaces(status);",
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
    vout: u64,
    height: u64,
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
                "SELECT COALESCE(MAX(height) + 1, 0) FROM namespaces;",
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
