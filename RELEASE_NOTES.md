# Release Notes - 0.3.0

## Highlights

Version 0.4.0 brings a few new features:

- [NOM-04](https://github.com/ursuscamp/noms/blob/master/nom-04.md) support: indexes are now publised to relays!
- `rebroadcast` CLI command which rebroadcasts all known records events to relays (useful to keep indexer network healthy)
- `publish` command will publish the full set of indexed names to relays

## Upgrading from 0.3

1. Backup your `nomen.db` file prior to upgrading.
2. Repalce your `nomen` executable.
3. Update your `nomen.toml` config file with the following new keys under the `[nostr]` section (if you wish to publish your index):
   1. `secret = "nsec..."` is the `nsec` encoded private key that your indexer will use to publish events
   2. `publish = true` will tell your node to publish index events to your Nostr relays
   3. `well-known = true` will make sure the indexer serves the `.well-known/nomen.json` file per the NOM-04 specification
4. Run `nomen publish` to publish a full index (again, only if you wish to publish)