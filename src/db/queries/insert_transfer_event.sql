INSERT INTO transfer_events (nsid, pubkey, created_at, event_id, name, content, indexed_at)
VALUES (?, ?, ?, ?, ?, ?, unixepoch())
ON CONFLICT(nsid) DO UPDATE SET
created_at = excluded.created_at, event_id = excluded.event_id;