SELECT re.records FROM blessed_blockchain_vw b
JOIN name_nsid nn ON b.nsid = nn.parent
JOIN create_events ce ON b.nsid = ce.nsid
JOIN records_events re on nn.nsid = re.nsid AND nn.pubkey = re.pubkey
WHERE re.nsid = ?;