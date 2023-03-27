SELECT nn.name, nn.nsid FROM blessed_blockchain_vw b
JOIN name_nsid nn ON b.nsid = nn.parent
JOIN create_events ce on b.nsid = ce.nsid
WHERE nn.parent = ? AND nn.name <> ce.name
ORDER BY nn.name;