# Nomen

Nomen is a protocol for deriving globally unique names based on Bitcoin and Nostr.

## Overview

This protocol is intended to be extremely simple. Bitcoin is used for ordering of claims to names, and Nostr is the data transport layer for name information.

Each name is a globally unique value, and are claimed on a first come/first serve basis. In order to claim (or register) a name, a claim is published to the Bitcoin blockchain in the form of am OP_RETURN. A special event representing the "record set" of key values is published to Nostr.

Indexers link on-chain claims to published Nostr events and provide an interface to query records for these names.

## Details

### Blockchain

Obtaining a name requires claiming an unused name on the Bitcoin blockchain by publishing it in an `OP_RETURN` output. A generalized output is of the `OP_RETURN NOM 0x01 <TRANSACTION TYPE> <TRANSACTION DATA>`. All data should be encoded as byte strings. The spaces here are for illustration purposes only and should be included in the final string.

`NOM` is a 3 byte tag to alert the indexer we may dealing with a Nomen claim.

`0x01` is a version byte.

`TRANSACTION TYPE` should be `0x00` for a claim to a new name, `0x01` for a transfer of a name, or `0x02` for a signature.

`TRANSACTION DATA` depends on the transaction type (see next sections).

**Note:** The owner of the Bitcoin UTXO that generated the `OP_RETURN`, or the amount in the UTXO, do not matter. Bitcoin, in this case, is being utilized only as a decentralized timestamp server. The only thing that matters is the order of transaction outputs.

### Creating a name

In order to claim an unsued name, publish an `OP_RETURN` with the following data: `OP_RETURN NOM 0x01 0x00 <PUBLIC KEY> <NAME>`.

The public key is the 32-byte Schnorr X-Only public key seen in Bitcoin and Nostr.

The remainder of the OP_RETURN may be taken by the name, as a bytes string. It can be up to 43-bytes.

If a claim is made to a name after the first initial claim, it is invalid.

### Transferring a name

Transferring a name requires publishing two Bitcoin transactions with an `OP_RETURN` output in each.

The first output should take this form: `OP_RETURN NOM 0x01 0x01 <NEW PUBLIC KEY> <NAME>`. It is similar to a create, but the key is the public key of the new owner.

The second output should take this form: `OP_RETURN NOM 0x01 0x02 <SCHNORR SIGNATURE>`. The Schnorr signature is the 64-byte signature signed by the **previous owner** of the name owner. The signed message is of the dummy Nostr event:

```json
{
  "pubkey": "previous owner pubkey",
  "created_at": 1,
  "kind": 1,
  "tags": [],
  "content": "<NEW OWNER PUBLIC KEY><NAME>"
}
```

`NEW OWNER PUBLIC KEY` is the lower-case hex encoded public key of the new owner. `NAME` is of course the name of being transferred. These two values should be concatenated together in the `content` of the event.

This event should be serialized and signed like any Nostr event, and the 32-byte value of the signature should be embedded in the `SCHNORR SIGNATURE` field.

This event **should not** be broadcast. The format of a Nostr event for the signature was chosen so that these `OP_RETURN` outputs could be generated in browsers and signed by a user's [NIP-07](https://github.com/nostr-protocol/nips/blob/master/07.md)-compliant Nostr browser extension.

When encountering a transfer output, an indexer should cache the transfer. When encountering a signature output, the indexer should scan the transfer cache for matching transfers. If the signature matches any transfers, that transfer becomes valid.

### Legacy protocol v0 names

The initial version of the protocol, v0, had a different name format. Only a limited number of these names were issued. The format for a claim looked like this: `OP_RETURN NOM 0x00 0x00 <NAME FINGERPRINT> <NAMESPACE ID>`.

The `NAME FINGERPRINT` was the fist five bytes of the HASH160 of the name. HASH160 is the RIPEMD-160 hash of a SHA-256 hash, just like in Bitcion.

The `NAMESPACE ID` was the HASH160 of the `<NAME><PUBLIC KEY>`.

To fully establish ownership, the index needed to connect a signed Nostr event with the on-chain claim. This was less than ideal for data availability reasons, thus the move from v0 to v1.

However, v1 is backwards compatible as v0 names are still supported in the same namespace. If the owner of a v0 name wishes to update to v1, perhaps to take advantage of a transfer, they need only post a v1 CREATE (`0x01`) transaction with the same name and pubkey used in the `NAMESPACE ID`.

When encountering a v1 `0x00` transaction, the indexer must calculate the `FINGERPRINT` and `NAMESPACE ID` of a v0 transaction, look for any matching existing names, and automatically upgrade the existing name to v1 if it exists.

If no such v0 name already exists, then the v1 `0x00` is appended as a new name as described above.

### Nostr

Nostr is the propogation layer of the protocol. The only required information on-chain is the information necessary to determination ownership of a name.

There is one new kind of Nostr event. It is a parameterized replaceable event (all events are idempotent and thus replaceable).

| Event kind | Event type    | Description                                                   |
|------------|---------------|---------------------------------------------------------------|
| 38300      | NAME          | Matches `0x00` tranaction type. Publishes records for a name. |

#### New Name

After publishing a `0x00` name transaction, publish a `38300` kind Nostr event. The `d` tag for the event should be the lower case hex representation of the `NAMESPACE ID`. Additionally, there should be a `nom` tag with the `name` value as the parameter. `content` must be a JSON-serialized object of key/value pairs. These key/value pairs represent the records for the name. For example, `NPUB` might be the owner's Nostr npub, `EMAIL` might be the owner's email, etc. See `Appendix D` for some recommended key types.

When the records need to be updated, the owner may just publish another name event with different records and it will be replaced.

When receiving new events, an indexer should recalculate the namespace ID and compare to the `d` tag to validate the event. Valid records for a name can only be accepted when published by the name's owner.

## Appendix A: Name format

It is necessary to limit the characters used in names. While it might be tempting to allow any valid UTF-8 string, there are good reasons not to do this. In the Unicode standards, there are sometimes different ways to the construct the same character, invisible characters, or "whitespace" characters that may not necessarily be rendered, etc. This could allow for malicious individuals to trick unsuspecting users into clicking/pasting incorrect names.

While it is desirable to have a wide range of characters and languages be usable, for the time being it is necessary to restrict the use of characters to the basic characters typically used in domain names today.

Names must match the following regular expression `[0-9a-z\-]{3,43}` and must be ignored by indexers otherwise.

## Appendix B: Protocol expansion

In the event of backward incompatible changes to the protocol (such as character expansions mentioned in `Appendix A`), it would be preferrable to set an activation blockheight where this feature becomes available. This should curb anyone attempting to "front run" a protocol expansion by registering new names early, before a feature is available.


## Appendix C: Recommended Records

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

**2023-09-23**:
  - Protocol v0 is now deprecated in favor of protocol v1. V1 is much simpler to track on chain, and much simpler to create indexers.
  - Squatting section removed as it is not related to this spec.

**2023-09-19**:
  - Given the issues cited [here](https://github.com/ursuscamp/nomen/issues/6), the design of transfers in `0x00` is not good and has been removed. As of the time of publication, no transfers have been issued by any users, so this is a not a breaking change.
  - A new version will be issued which will enable transfers, and an upgrade path will be available for version `0x00` names to `0x01` names. However, in order to link the `0x00` to `0x01` names on chain, the upgrade transaction will require the names to be put on chain in plain text, necessitating limiting them to a maximum of 43 bytes for now (80 byte OP_RETURN maximum - 5 bytes for Nomen metadata - 32 bytes for a public key). As of the time of publication, no names longer than 43 bytes have been issued, so this is a non-breaking change.

