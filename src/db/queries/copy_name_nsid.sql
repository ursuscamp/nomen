insert into name_nsid (name, nsid, parent, pubkey)
select nn.name, nn.nsid, ?, nn.pubkey
from name_nsid nn
where nn.parent = ?;