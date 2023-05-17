# Nomen Explorer

Nomen is a protocol for globally unique, decentralized "domain names". The Nomen Explorer is the first indexer (or name server) for this protocol.

Try it [here](https://nomenexplorer.com)! You can explore existing names or create a new one. Note: You will need to sign and broadcast a Bitcoin transaction with your wallet to do it.

If you download the project yourself, you can build it and run the indexer for your own use, or use the CLI to experiment with Nomen.

## What is Nomen?

Nomen is a protocol for globally unique names, like DNS, except decentralized and based on Bitcoin and Nostr. Instead of a central authority deciding who controls a name, the protocol provides simple rules to determine the owner.

At a high level, claims to a name are published to the Bitcoin blockchain (think of this as registering a domain name). Bitcoin provides the ordering guarantees. The first to claim a name owns it. Published along with the name is the public key of the owner.

Owners then publish Nostr events signed with the same key to update their records (like their domain DNS records).

With Bitcoin, there is no need to create a new blockchain or have a trusted third party. With Nostr, there's no need to bootstrap a new P2P transport layer.

Read [the spec](https://github.com/ursuscamp/nomen/blob/master/docs/SPEC.md) for more details about the protocol itself. It's very simple.