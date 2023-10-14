# Frequently Asked Questions

## What is Nomen?

Nomen is a protocol to create a name system akin to DNS, but open, permissionless and self-sovereign. It's open, because it's all developed in the open. It's permissionless, because no one can stop you from having a name. It's self-sovereign because you can run it yourself, so no one can gatekeep your access to the protocol.

## Why Bitcoin? Why Nostr?

Blockchains provide decentralized guarantees on the ordering of events. One thing that needs those guarantees is money, but another thing could be argued to be identity. In this case, human-readable identity. Human-readable globally unique names are a limited quantity. In order to hand them out fairly, you traditionally needed a central authority like ICANN. But central authorities are corruptible. They can censor. The Bitcoin network is the most secure decentralized ordering mechanism on earth. We can use this to order claims to identities in the most fair way possible: first come, first serve.

However, not all data can reasonably fit on the blockchain. The limited space means that a secondary protocol is needed to distribute the majority of the data. This is where Nostr comes in. Nostr is an open protocol with existing network effects that is perfect for this.

## How does it work?

It's actually very simple. Claims to identities are published to the Bitcoin blockchain. The claims are very small, only a single OP_RETURN output (80 bytes or less). Events are then published to Nostr relays that contain teh records you want to associate with your name (npub, web address, twitter handle, etc). Per the protocol, even if two individuals claim the same identifier, the Bitcoin blockchain guarantees that one will be first, and thus valid.

## Why not inscriptions?

There are two parts of inscription that might have been used: Sat tracking and the inscription envelope.

Sat tracking could have been used to allow transferring the name, just like Ordinals inscriptions. This adds complexity for the end user and developer because then you have to build a wallet with special coin tracking, or make sure that the user understands to be extremely careful with their UTXOs. Additionally, this is intended to be a "Nostr-native" protocol, and that means it's the Bitcoin keys that own the name, not Nostr keys. While it's true that Nostr keys can be used as Bitcoin keys in taproot addresses, all of the added complexity of trying to make it all work wasn't worth it.

Inscription envelopes could have been useful. Instead of using OP_RETURN for putting the data on chain, we could have used an inscription envelope containing the name and ownership info. But inscription envelopes are really more useful for stuffing lots of data on chain. As it stands, OP_RETURN allows enough space for Nomen to include a pubkey for ownership data, and still leave 43 bytes for the name. Unlike NFTs, no one WANTS a long name. Short names are more desirable, so OP_RETURN is not only much simpler (inscriptions always require two transactions), but it is also the "official" and blessed way of putting data on chain.