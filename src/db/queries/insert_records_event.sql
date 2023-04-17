INSERT INTO records_events (pubkey, created_at, event_id, name, records, indexed_at)
VALUES (?, ?, ?, ?, ?, unixepoch())
ON CONFLICT (name, pubkey) DO UPDATE SET
created_at = excluded.created_at,
event_id = excluded.event_id,
records = excluded.records
where excluded.created_at > created_at;