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

The `NAMESPACE_ID` is a 20-byte hash value of the namespace data. It is created by concatenating the following values: 32-byte secp256k1 public key, 32-byte merkle root for namespace data, and the UTF-8 encoded __fully qualified name__. Then the value is run through the HASH160 algorithm like so: `RIPEMD160(SHA256(PUBKEY MERKLE_ROOT? FULLY_QUALIFIED_NAME))`. The 20-byte value produced by this function is the `NAMESPACE_ID`, which uniquely identifies a root name.

If a namespace has no children, the merkle root should be skipped.

The fully qualified name is a complete name, from parent's name all the way to a child. So if the top-level namespace `com` has child `amazon` which in turn has a child `www` then the fully qualified name is `www.amazon.com`.

Namespace names's must obey the following limitations:

* Must be 256 bytes or less when fully qualified (restriction may be lifted in later versions).
* Must be match the following regular expression: `[a-z][a-z0-9\-_]*` (restrictions may be lifted in later versions).
* Must NOT begin with underscore character `_`. All names beginning with underscore characters are reserved for protocol use. Even when character restrictions are lifted, `_` will always be reserved.

During indexing, a client must ignore any transactions where any names violate these rules. For the purposes of the protocol, the invalid transaction may as well not exist. In the case of an initiation transaction, for example, if a namespace root is valid, but a child name is not valid, the entire transaction is invalid, and the namespace root is still considered "free" to be claimed by anyone.


#### Index

##### Merkle Roots

Each domain makes up a namespace of child namespaces, potentially infinitely. Each commitment to the blockchain has a namespace ID (`nsid`), as mentioned above. Part of the __NSID__ is a merkle root who's tree consists of the child namespaces __NSIDs__. The merkle root is calculated by doing a HASH160 on a __NSID__ and it's neighbor. If there are an odd number of names, then the last one is hashed with itself. When there is only one hash remaining, this is the merkle root.

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