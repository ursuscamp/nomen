SELECT blockhash, txid, vout, blockheight
FROM blockchain b
WHERE nsid = ?;