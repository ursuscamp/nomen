# Release Notes - 0.3.0

## Highlights

Version 0.3.0 is a massive change. The banner feature of this new version is a protocol version bump to v1 which:

- puts all ownership data on chain
- backwards-compatible with v0
- upgradeable from v0
- enables transfer
- long-term stable

See the changelog for a full list of changes.

## Upgrading from 0.2

This version involves a full database schema revamp, so backup your `nomen.db` file, then delete it. Upgrading will require a full blockchain rescan!

Checkout the new [example.nomen.toml](example.nomen.toml) file and adjust your configuration accordingly for the new version.