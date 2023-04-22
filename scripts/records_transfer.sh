#!/usr/bin/env bash
CREDS="-regtest -rpcuser=regtest -rpcpassword=regtest"
CMD="bitcoin-cli $CREDS -rpcwallet=regtest"

PRIVKEY="23dfe0450af72a460acb5322372b43265885facca7b2539bb8c568c432068820"
PUBKEY="cb5dd62f5018ddd6dc9c49b492b20c78dc3c84fc7f237b101334c5aed2bb6247"

RUST_LOG=off cargo run -- name record --privkey $PRIVKEY smith IP4=transfer NPUB=transfer