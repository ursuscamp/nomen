INSERT INTO
  blockchain_index (
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
  (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch());