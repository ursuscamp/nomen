INSERT INTO records_events (nsid, pubkey, created_at, event_id, name, records)
VALUES (?, ?, ?, ?, ?, ?)
ON CONFLICT (nsid, pubkey) DO UPDATE SET
created_at = excluded.created_at,
event_id = excluded.event_id,
records = excluded.records
where excluded.created_at > created_at;