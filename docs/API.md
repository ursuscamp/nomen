# Nomen Indexer REST API

The Nomen indexer has a rest API that some may find useful. This API is currently experimental and subject to change.

## Errors

All methods return `200` on success, otherwise they return `400` on an error, which the JSON response:

```json
{
  "error": "<ERROR MESSAGE>"
}
```

## Methods

### `GET /api/names`

List of indexed names and owners.

**Request Type**: `Query Params`

**Request Body**: `N/A`

**Response Type**: `JSON`

**Response Body**:

```json
[
  {
    "name": "",
    "pubkey": ""
  }
]
```

### `GET /api/name`

Queries information and metadata about a specific name.

**Request Type**: `Query Params`

**Request Body**: `name` is a string parameter matching the name to query.

**Response Type**: `JSON`

**Response Body**:

```json
[
  {
    "records": {}
  }
]
```

### `GET /api/create/data`

Returns a valid `OP_RETURN` which can be included in a Bitcoin transaction to claim a particular name.

**Request Type**: `Query Params`

**Request Body**: `name` is a string parameter matching the name to query. `pubkey` is the hex-encoded X-Only public key of the name's owner.

**Response Type**: `JSON`

**Response Body**:

```json
[
  {
    "op_return": ["<OP_RETURN VALUE>"]
  }
]
```

### `GET /api/transfer/event`

Returns an unsigned Nostr event which is used as a standard wrapper format for transfer signatures. This event must be signed by **current** owner of the name. This event may be signed like any Nostr event, then the `sig` field can be isolated and used as an on-chain signature for the transfer.

**Request Type**: `Query Params`

**Request Body**: `name` is a string parameter matching the name to query. `new_owner` is the hex-encoded X-Only public key of the name's new owner. `old_owner` is the hex-encoded X-Only public key of the name's current owner.

**Response Type**: `JSON`

**Response Body**:

```json
[
  {
    "event": {}
  }
]
```

### `GET /api/transfer/data`

Returns two valid `OP_RETURN` which can be included in a Bitcoin transactions to claim a particular name. The first `OP_RETURN` caches the transfer, and the second `OP_RETURN` is the signature.

**Request Type**: `Query Params`

**Request Body**: `name` is a string parameter matching the name to query. `new_owner` is the hex-encoded X-Only public key of the name's new owner. `signature` is the `sig` field of the signed Nostr transfer event (for example, returned by `GET /api/transfer/event`).

**Response Type**: `JSON`

**Response Body**:

```json
[
  {
    "op_return": ["<OP_RETURN TRANSFER>", "<OP_RETURN SIGNATURE>"]
  }
]
```