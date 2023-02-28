# GUNS

## Global Uncensorable Name System

### Specification

#### Introduction

The following lay out the specification for GUNS, the Global Uncensorable Name System. It is designed in the hopes that it will provide a global name system without the need for a centralized organization like ICANN with final control authority.

#### Workflow

##### Initiation Transaction

In order to initiate a new namespace, a `NAMESPACE_ID` must be commited to the Bitcoin blockchain in an OP_RETURN output (output value may be `0`). This is called an __initiation transaction__.

The initiation transaction must take this form: `OP_RETURN GUN \x00 \x00 NAMESPACE_ID`.

The null byte `\x00` following `GUN` is a version byte. It's made to indicate which version of the parser to user for the following information. Currently, version `0` is the only version, but potential future protocol revisions should increment by 1.

The next null byte is a __transaction type__. The transaction type indicates indicates which transaction will follow. Transsaction type `0` indicates a namespace intitiation transaction.

The `NAMESPACE_ID` is a 20-byte hash value of the namespace data. It is created by concatenating the following values: 32-byte secp256k1 public key, 32-byte merkle root for namespace data, and the UTF-8 encoded __namespace root__, or what we would typically think of a top-level name. Then the value is run through the HASH160 algorithm like so: `RIPEMD160(SHA256(PUBKEY MERKLE_ROOT NAMESPACE_ROOT))`. The 20-byte value produced by this function is the `NAMESPACE_ID`, which uniquely identifies a root name.

Namespace names's must obey the following limitations:

* Must be 256 bytes or less (restriction may be lifted in later versions).
* Must be valid UTF-8 strings.
* Must NOT begin with underscore character `_`. All names beginning with underscore characters are reserved for protocol use.

During indexing, a client must ignore any transactions where any names violate these rules. For the purposes of the protocol, the invalid transaction may as well not exist. In the case of an initiation transaction, for example, if a namespace root is valid, but a child name is not valid, the entire transaction is invalid, and the namespace root is still considered "free" to be claimed by anyone.


#### Index

##### Transaction Types

| Transaction Type | Description            |
|------------------|------------------------|
| 0                | Initiation Transaction |

##### Standard Name Types

| Name Type | Description      |
|-----------|------------------|
| 4         | IPv4 address     |
| 6         | IPv6 address     |
| NPUB      | Nostr public key |