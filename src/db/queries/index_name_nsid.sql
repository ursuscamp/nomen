INSERT INTO name_nsid (name, nsid, root, parent, pubkey) VALUES (?, ?, ?, ?, ?)
ON CONFLICT DO NOTHING;