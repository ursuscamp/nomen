INSERT INTO name_events (name, fingerprint, nsid, pubkey, created_at, event_id, records, indexed_at, raw_event)
VALUES (?, ?, ?, ?, ?, ?, ?, unixepoch(), ?)
ON CONFLICT (name, pubkey) DO UPDATE SET
created_at = excluded.created_at,
event_id = excluded.event_id,
records = excluded.records,
raw_event = excluded.raw_event
where excluded.created_at > created_at;