SELECT nn.name, nn.nsid FROM blockchain b
JOIN name_nsid nn ON b.nsid = nn.root
JOIN create_events ce on b.nsid = ce.nsid
WHERE nn.parent = ?
ORDER BY nn.name;