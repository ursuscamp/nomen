#!/usr/bin/env bash
CREDS="-regtest -rpcuser=regtest -rpcpassword=regtest"
CMD="bitcoin-cli $CREDS -rpcwallet=regtest"

OLD_PRIVKEY="f5daf17ccf02488bc0ab506fc550016963af3030d4c5d2b7b3e3c232f3c0d7ca"
OLD_PUBKEY="d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f"

NEW_PRIVKEY="23dfe0450af72a460acb5322372b43265885facca7b2539bb8c568c432068820"
NEW_PUBKEY="cb5dd62f5018ddd6dc9c49b492b20c78dc3c84fc7f237b101334c5aed2bb6247"

ADDR=$($CMD getnewaddress)
FUNDED_PSBT=$($CMD walletcreatefundedpsbt '[]' "[{\"$ADDR\":1}]" 0 "{\"fee_rate\": 5}" | jq -r .psbt)
DATA=$(RUST_LOG=off cargo run -q -- name transfer --privkey $OLD_PRIVKEY --json --broadcast --validate smith $NEW_PUBKEY $FUNDED_PSBT)
UNSIGNED_PSBT=$(echo $DATA | jq -r .unsigned_tx)
SIGNED_PSBT=$($CMD walletprocesspsbt $UNSIGNED_PSBT | jq -r .psbt)
SIGNED_TX=$($CMD finalizepsbt $SIGNED_PSBT | jq -r .hex)
$CMD sendrawtransaction $SIGNED_TX
$CMD generatetoaddress 3 $ADDR
# RUST_LOG=off cargo run -q -- util sign-event --privkey $OLD_PRIVKEY --broadcast $UEVENT