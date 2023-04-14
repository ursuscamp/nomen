#!/usr/bin/env bash
CREDS="-regtest -rpcuser=regtest -rpcpassword=regtest"
CMD="bitcoin-cli $CREDS -rpcwallet=regtest"

PRIVKEY="f5daf17ccf02488bc0ab506fc550016963af3030d4c5d2b7b3e3c232f3c0d7ca"
PUBKEY="d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f"

RUST_LOG=off cargo run -- name record --privkey $PRIVKEY smith IP4=127.0.0.1 NPUB=npub1234