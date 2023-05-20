#!/usr/bin/env bash
CREDS="-regtest -rpcuser=regtest -rpcpassword=regtest"
CMD="bitcoin-cli $CREDS -rpcwallet=regtest"

PRIVKEY="f5daf17ccf02488bc0ab506fc550016963af3030d4c5d2b7b3e3c232f3c0d7ca"
PUBKEY="d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f"

ADDR=$($CMD getnewaddress)
FUNDED_PSBT=$($CMD walletcreatefundedpsbt '[]' "[{\"$ADDR\":1}]" 0 "{\"fee_rate\": 5}" | jq -r .psbt)
UNSIGNED_PSBT=$(RUST_LOG=off cargo run -q -- name new --privkey $PRIVKEY --json --broadcast smith $FUNDED_PSBT | jq -r .unsigned_tx)
SIGNED_PSBT=$($CMD walletprocesspsbt $UNSIGNED_PSBT | jq -r .psbt)
SIGNED_TX=$($CMD finalizepsbt $SIGNED_PSBT | jq -r .hex)
$CMD sendrawtransaction $SIGNED_TX
$CMD generatetoaddress 3 $ADDR