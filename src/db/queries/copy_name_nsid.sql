insert into name_nsid (name, nsid, parent, pubkey, child)
select nn.name, nn.nsid, ?, nn.pubkey, child
from name_nsid nn
where nn.parent = ?;