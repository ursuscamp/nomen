INSERT INTO
  raw_blockchain (
    blockhash,
    txid,
    blocktime,
    blockheight,
    txheight,
    vout,
    data,
    indexed_at
  )
VALUES
  (?, ?, ?, ?, ?, ?, ?, unixepoch());