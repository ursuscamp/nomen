use bitcoin::BlockHash;
use nomen_core::Nsid;
use secp256k1::XOnlyPublicKey;
use sqlx::SqlitePool;

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

pub async fn update_index_for_transfer(
    conn: &sqlx::Pool<sqlx::Sqlite>,
    nsid: Nsid,
    new_owner: XOnlyPublicKey,
    old_owner: XOnlyPublicKey,
    name: String,
) -> Result<(), anyhow::Error> {
    sqlx::query("UPDATE blockchain_index SET nsid = ?, pubkey = ? WHERE name = ? AND pubkey = ?;")
        .bind(hex::encode(nsid.as_ref()))
        .bind(hex::encode(new_owner.serialize()))
        .bind(&name)
        .bind(hex::encode(old_owner.serialize()))
        .execute(conn)
        .await?;
    Ok(())
}
