INSERT INTO name_events (nsid, pubkey, created_at, event_id, name, content)
VALUES (?, ?, ?, ?, ?, ?)
ON CONFLICT(nsid) DO UPDATE SET
created_at = excluded.created_at, event_id = excluded.event_id;