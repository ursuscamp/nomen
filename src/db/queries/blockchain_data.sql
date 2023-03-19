SELECT blockhash, txid, vout, height
FROM blockchain b JOIN name_nsid nn ON b.nsid = nn.root
WHERE nn.nsid = ?;