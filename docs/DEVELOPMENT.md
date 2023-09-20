# Setting up a dev environment

1. Clone the repo.
2. Create a branch from `develop`.

## Bitcoin

1. Start bitcoin in regtest: `make bitcoin-local`. This sets up a local Bitcoin regtest environment just for Nomen.
   - If you ever need to reset your local regtest: Stop `bitcoin` and run `make bitcoin-reset`.
2. Run `make bitcoin-wallet` to setup the default wallet for Bitcoin.
3. Create an alias like `regtest` to `bitcoin-cli -datadir=.bitcoin/ -chain=regtest`.

## Nostr Relay

1. In a separate folder, clone `https://github.com/scsibug/nostr-rs-relay`.
2. Run `cargo build --release`.
3. Run `RUST_LOG=info target/release/nostr-rs-relay`.
   - This will start a local Nostr relay for Nomen to use.
   - If you ever need to reset, just `rm nostr.db` and run the command again.

## Nomen

Back in your Nomen folder:

1. Copy [development.nomen.toml](./development.nomen.toml) to `nomen.toml` in the root folder.
2. Run `cargo run -- server` to start the Nomen indexer.