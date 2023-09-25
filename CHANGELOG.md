# Changelog

## 0.3.0 (Unreleased)

Features:

Bugs:

Other:
  - Set a minimum supported Rust version of 1.71. Will try to maintain this for a while.
  - Refactored into a Cargo multi-crate workspace. `nomen_core` contains all types, `nomen` contains the indexer/http server and `nomen-cli` contains command-line utilities.

## 0.2.0

This release includes a database migration, so make sure to back up your index before upgrading.

Features:
  - Transfers have been removed, and names have been limited to 43 characters for vesion `0x00`. They will be enabled in the next version with a better designed.
  - Primal.net is now used to npub links.
  - New page to list blockchain claims for which there are no indexed record events.
  - Index statistic page.

Bugs:
  - Fixed a bug where a name was double-indexed because the same `OP_RETURN` was uploaded twice

## 0.1.1

Features:
  - Explorer now links to a name instead of a NSID. This simply makes it easier for a something to be bookmarked, even after a transfer.
  - Explorer web UI and CLI both automatically capitalizes the keys in records now.
  - Name page: Update Records link added, which automatically preloads data for user to update, including most recent record set.
  - Name page: Blockhash and Txid link to block explorer mempool.space.
  - Name page: Links for different record types. For example, `WEB` record links to actual webpage.
  - Name page: MOTD records now have a little but of decorative quoting.
  - The Search bar strips whitespace.

Bugs:
  - Indexer will not longer stop randomly.

Other:
  - Added `WEB` record type to spec.
  - Changes "New Records" to "Update Records" everywhere.
  - More detailed help instructions.

## 0.1.0

- Initial release.