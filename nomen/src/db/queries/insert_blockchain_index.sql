INSERT INTO
  blockchain_index (
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