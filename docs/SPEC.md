# Nomen

Nomen is a protocol for deriving globally unique names based on Bitcoin and Nostr.

## Overview

This protocol is intended to be extremely simple. Bitcoin is used for ordering of claims to names, and Nostr is the data transport layer for name information.

Each name is a globally unique value, and are claimed on a first come/first serve basis. In order to claim (or register) a name, a claim is published to the Bitcoin blockchain in the form of a hash in an OP_RETURN. A special event is published to Nostr with the necessary information to recreate the hash.

Indexers (like a DNS name server) link on-chain claims to published Nostr events and provide an interface to query records for these names.

## Details

### Blockchain

In order to claim a name, publish an output to the bitcoin blockchain in this format: `OP_RETURN NOM <VERSION> <TRANSACTION TYPE> <NAME FINGERPRINT> <NAMESPACE ID>`. The spaces are for readability and not significant. Do not include spaces in the final `OP_RETURN`. All names have an associated owner, which just a private/public keypair. Public keys are always 32-byte X-Only Public Keys used everywhere in Nostr and Bitcoin Schnorr signatures.

`NOM` is just the byte string "NOM", a tag indicator to let the indexer know this ia Nomen output.

`VERSION` is reserved for future use, for incompatible changes to the protocol, or unlocking additional namespace. It is one byte and must currently be `0x00`.

`TRANSACTION TYPE` represents the type of claim being made on chain. It is one byte. It may be `0x00` which represents a new name being claimed, or `0x01` which represents an ownership change of the name (owned by a new keypair).

`NAME FINGERPRINT` is the first five bytes of the HASH-160 of the name. The purpose is to allow a name to be verified as unreserved, even if a Nostr event cannot be found to prove it.

`NAMESPACE ID` represents a HASH-160 (20-byte) hash of the ownership information for this name. If the `TRANSACTION TYPE` is `0x00` (new name) then the `NAMESPACE ID` is the HASH-160 of `<NAME><OWNER PUBKEY>`. Please be aware that the pubkey is a 32-byte byte string of the pubkey, not any textual representation such as bech32 or hexadecimal.

**Note:** The owner of the Bitcoin UTXO that generated the `OP_RETURN`, or the amount in the UTXO, do not matter. Bitcoin, in this case, is being utilized only as a decentralized timestamp server. The only thing that matters is the order of transaction outputs.

### Nostr

Nostr is the propogation layer of the protocol. The only required information on-chain is the information necessary to determination ownership of a name, and that is only in a shortened hash form.

There is one new kind of Nostr event. It is a parameterized replaceable event (all events are idempotent and thus replaceable).

| Event kind | Event type    | Description                                                   |
|------------|---------------|---------------------------------------------------------------|
| 38300      | NAME          | Matches `0x00` tranaction type. Publishes records for a name. |

#### New Name

After publishing a `0x00` name transaction, publish a `38300` kind Nostr event. The `d` tag for the event should be the lower case hex representation of the `NAMESPACE ID` published to the blockchain. Additionally, there should be a `nom` tag with the `name` value as the parameter. `content` must be a JSON-serialized object of key/value pairs. These key/value pairs represent the records for the name. For example, `NPUB` might be the owner's Nostr npub, `EMAIL` might be the owner's email, etc. See `Appendix D` for some recommended key types.

When the records need to be updated, the owner may just publish another name event with different records and it will be replaced.

**Note:** When receiving new events, and indexer should recalculate the namespace ID and compare to the `d` tag to validate the event, then use the namespace ID to link to blockchain for correct ordering. Indexers should also treat any blockchain transactions with mis-matching name fingerprints as invalid.

## Appendix A: Name format

It is necessary to limit the characters used in names. While it might be tempting to allow any valid UTF-8 string, there are good reasons not to do this. In the Unicode standards, there are sometimes different ways to the construct the same character, invisible characters, or "whitespace" characters that may not necessarily be rendered, etc. This could allow for malicious individuals to trick unsuspecting users into clicking/pasting incorrect names.

While it is desirable to have a wide range of characters and languages be usable, for the time being it is necessary to restrict the use of characters to the basic characters typically used in domain names today.

Names must match the following regular expression `[0-9a-z\-]{3,43}` and must be ignored by indexers otherwise.

## Appendix B: Protocol expansion

In the event of backward incompatible changes to the protocol (such as character expansions mentioned in `Appendix A`), it would be preferrable to set an activation blockheight where this feature becomes available. This should curb anyone attempting to "front run" a protocol expansion by registering new names early, before a feature is available.

## Appendix C: Squatting

Squatting is definitely a problem in decentralized name systems. Some take it as a necessary evil, but under Nomen this is triviably solveable if it ever becomes a major issue. In the standard protocol, anyone can publish a claim to a name (by publishing an on-chain transaction and `38300` Nostr event). However, indexers will ignore additional claims after the first.

However, if it becomes well known that a certain name is held by a squatter, an index could choose to ignore claims in favor of later ones. If Bob claims the name `amazon`, but the real Amazon comes along later and registers a claim, many indexers may just choose to ignore the first claim by Bob in favor of the real Amazon.

However, this protocol is intended to be self-sovereign and censorship resistant, so any individual or organization may run their own indexer and use any such rules they wish.

In the future, a protocol addition may even include the ability for indexers to subscribe to spam lists (published as Nostr events) from trusted third parties which crowdsource the hard work of figuring out which individuals are squatters or malicious actors, similar to spam blockers.

## Appendix D: Recommended Records

There are no restrictions on key/value pairs, but they are recommended by convention for the key names to be uppercase. It would be useful to establish some standard keys by convention for simple interoperability. Here are suggested keys:

| KEY NAME | DESCRIPTION                                               |
|----------|-----------------------------------------------------------|
| `IP4`    | IPv4 address for a website                                |
| `IP6`    | IPv6 address for a website                                |
| `DNS`    | Alias for DNS hostname                                    |
| `NPUB`   | Nostr NPUB                                                |
| `EMAIL`  | Owner email address                                       |
| `MOTD`   | A general message from the owner                          |
| `WEB`    | Full link for website (not necessarily the same as `DNS`) |

Others may arise later by addition or general public acceptance. The above listed are not required, but if the owner wishes to include any of this data in their records, it is recommended to use the above keys.

## Appendix ZZ: Changes and Updates

**2023-09-19**:
  - Given the issues cited [here](https://github.com/ursuscamp/nomen/issues/6), the design of transfers in `0x00` is not good and has been removed. As of the time of publication, no transfers have been issued by any users, so this is a not a breaking change.
  - A new version will be issued which will enable transfers, and an upgrade path will be available for version `0x00` names to `0x01` names. However, in order to link the `0x00` to `0x01` names on chain, the upgrade transaction will require the names to be put on chain in plain text, necessitating limiting them to a maximum of 43 bytes for now (80 byte OP_RETURN maximum - 5 bytes for Nomen metadata - 32 bytes for a public key). As of the time of publication, no names longer than 43 bytes have been issued, so this is a non-breaking change.

