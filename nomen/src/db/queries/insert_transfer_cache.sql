INSERT INTO
  transfer_cache (
    protocol,
    fingerprint,
    nsid,
    name,
    pubkey,
    blockhash,
    txid,
    blocktime,
    blockheight,
    txheight,
    vout,
    indexed_at
  )
VALUES
  (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch());