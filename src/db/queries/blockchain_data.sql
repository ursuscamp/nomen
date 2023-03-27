SELECT blockhash, txid, vout, blockheight
FROM blessed_blockchain_vw b
JOIN name_nsid nn ON b.nsid = nn.parent
WHERE nn.nsid = ?;