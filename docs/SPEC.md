# Nomen

Nomen is a protocol for deriving globally unique names based on Bitcoin and Nostr.

## Overview

This protocol is intended to be extremely simple. Bitcoin is used for ordering of claims to names, and Nostr is as the data transport layer for name information.

Each name is a globally unique value, and are claimed on a first come/first serve basis. In order to claim (or register) a name, a claim is published to the Bitcoin blockchain in the form of a hash in an OP_RETURN. A special event is published to Nostr with the necessary information to recreate the hash.

Indexers (like a name server) link on-chain claims to published Nostr events and provide an interface to query records for these names.

## Details

### Blockchain

In order to claim a name, publish an output to the bitcoin blockchain in this format: `OP_RETURN <VERSION><TRANSACTION TYPE><NAMESPACE ID>`. All names have an associated owner, which just a private/public keypair. Public keys are always 32-byte X-Only Public Keys used everywhere in Nostr and Bitcoin Schnorr signatures.

`VERSION` is reserved for future use, for incompatible changes to the protocol, or unlocking additional namespace. It is one byte and must currently be `0x00`.

`TRANSACTION TYPE` represents the type of claim being made on chain. It is one byte. It may be `0x00` which represents a new name being claimed, or `0x01` which represents an ownership change of the name (owned by a new keypair).

`NAMESPACE ID` represents a HASH-160 (20-byte) hash of the ownership information for this name. If the `TRANSACTION TYPE` is `0x00` (new name) then the `NAMESPACE ID` is the HASH-160 of `<NAME><OWNER PUBKEY>`. If the `TRANSACTION TYPE` is `0x01` (ownership change), then the `NAMESPACE ID` is the HASH-160 of `<NAME><NEW OWNER PUBKEY>`.

**Note:** The owner of the Bitcoin UTXO that generated the `OP_RETURN`, or the amount of the from UTXO, do not matter. Bitcoin, in this case, is being utilized only as a decentralized timestamp server. The only thing that matters is the order.

### Nostr

Nostr is the propogation layer of the protocol. The only information on-chain is the information necessary to determination ownership of a name, and that is only in a shortened hash form.

There are currently three types of Nostr events. These are all parameterized replaceable events (all events are idempotent and thus replaceable).

| Event kind | Event type    | Description                                    |
|------------|---------------|------------------------------------------------|
| 38300      | NEW NAME      | Match to `0x00` transaction type               |
| 38301      | RECORDS       | Publish a new record set for a particular name |
| 38300      | TRANSFER NAME | Match to `0x01` transaction type               |

#### New Name

After publishing a `0x00` name transaction, publish a `38300` kind Nostr event. The `d` tag for the event should be the lower case hex representation of the `NAMESPACE ID` published to the blockchain. Additionally, there should be a `nom` tag with the `name` value as the parameter. `content` is unused, but recommended to be empty. The published event must be signed by the keypair that made the claim on the blockchain. I.e., the `pubkey` value should be part of the namespace ID.

When receiving new events, and indexer should recalculate the namespace ID and compare to the `d` tag to validate, then use the namespace ID to link to blockchain for correct ordering.

#### Records

Every name can represent a series of key/value pairs, much like DNS. These records can be any info the owner wishes to convey with their name. It could anything from an IP address/DNS name for a website, to an NPUB, email address, etc.

To update records, publish a `38301` event to Nostr. The `d` tag should be the `NAMESPACE ID`. The `nom` tag should have the value of the `name`. The event must be signed by the owning keypair. The `content` must be a JSON-encoded object representing the record key/value pairs. Because this is a replaceable event, updating records just means publishing a new record event.

**Note:** This event type does _NOT_ have an on-chain equivalent, as there is no ownership change involved here.

#### Transfer

After publishing a `0x01` transfer transaction, publish a `38302` kind Nostr event. The `d` tag for the event should be the lower case hex representation of the `NAMESPACE ID` published to the blockchain. Additionally, there should be a `nom` tag with the `name` value as the parameter. `content` must be lowercase hex encoded value of the pubkey of the **_new_** owner. The published event must be signed by the current owner of the name, in order to properly establish chain of custody.

When receiving new events, and indexer should recalculate the namespace ID and compare to the `d` tag to validate, then use the namespace ID to link to blockchain for correct ordering. Unlike publishing new names, the namespace ID in this case is not constructed from the pubkey of the original owner, but the pubkey of the **_new_** owner.

## Appendix A: Name format

It is necessary to limit the characters used in names. While it might be tempting to allow any valid UTF-8 string, there are good reasons not to do this. In the UNICODE standards, there are sometimes different ways to the construct the same character, invisible characters, or "whitespace" characters that may not necessarily be rendered, etc. This could allow for malicious individuals to trick unsuspecting users into clicking/pasting incorrect names.

While it is desirable to have a wide range of characters and languages be usable, for the time being it is necessary to restrict the use of characters to the basic characters typically used in domain names today.

Names must match the following regular expression `[0-0a-z\-]{3,256}` and must be ignored by indexers otherwise.

## Appendix B: Protocol expansion

In the event of backward incompatible changes to the protocol (such as character expansions mentioned in Appendix A), it would be preferred to set an activation blockheight where this feature becomes available. This should curb anyone attempting to "front run" a protocol expansion by registering new things early, before a feature is available.

## Appendix C: Squatting

Squatting is definitely a problem in decentralized name systems. Some take it as a necessary evil, but under Nomen this is triviably solveable if it ever becomes a major issue. In the standard protocol, anyone can publish a claim to a name (by publishing an on-chain transaction and `38300` Nostr event). However, indexers will ignore additional claims after the first.

However, if it becomes well known that a certain name is held by a squatter, an index could choose to ignore claims in favor of later ones. If Bob claims the name `amazon`, but the real Amazon comes along later and registers a claim, many indexers may just choose to ignore the first claim by Bob in favor of the real Amazon.

However, this protocol is intended to be self-sovereign and censorship resistant, so any individual or organization may run their own indexer and use any such rules they wish.

In the future, a protocol addition may even include the ability for indexers to subscribe to spam lists (published as Nostr events) from trusted third parties which crowsource the hard work of figuring out which individuals are squatters, similar to spam blockers.