SELECT re.records FROM blockchain b
JOIN name_nsid nn ON b.nsid = nn.root
JOIN create_events ce ON b.nsid = ce.nsid
JOIN records_events re on nn.nsid = re.nsid AND nn.pubkey = re.pubkey
WHERE re.nsid = ?;