# Frequently Asked Questions

## What is Nomen?

Nomen is a protocol to create a name system akin to DNS, but open, permissionless and self-sovereign. It's open, because it's all developed in the open. It's permissionless, because no one can stop you from having a name. It's self-sovereign because you can run it yourself, so no one can gatekeep your access to the protocol.

## Why Bitcoin? Why Nostr?

Blockchains provide decentralized guarantees on the ordering of events. One thing that needs those guarantees is money, but another thing could be argued to be identity. In this case, human-readable identity. Human-readable globally unique names are a limited quantity. In order to hand them out fairly, you traditionally needed a central authority like ICANN. But central authorities are corruptible. They can censor. The Bitcoin network is the most secure decentralized ordering mechanism on earth. We can use this to order claims to identities in the most fair way possible: first come, first serve.

However, not all data can reasonably fit on the blockchain. The limited space means that a secondary protocol is needed to distribute the majority of the data. This is where Nostr comes in. Nostr is an open protocol with existing network effects that is perfect for this.

## How does it work?

It's actually very simple. Claims to identities are published to the Bitcoin blockchain. The claims are very small, only hashes. The data in the hash is enough to verify ownership (name + public key). This hash is the "nsid" or namespace identifier. Events are then published to Nostr relays that contain the data necessary to reconstruct the hash, and the events are signed by the same public key in the ownership hash. Per the protocol, even if two individuals claim the same identifier, the Bitcoin blockchain guarantees that one will be first, and thus valid.