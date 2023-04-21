#!/usr/bin/env bash
CREDS="-regtest -rpcuser=regtest -rpcpassword=regtest"
CMD="bitcoin-cli $CREDS -rpcwallet=regtest"

PRIVKEY="f5daf17ccf02488bc0ab506fc550016963af3030d4c5d2b7b3e3c232f3c0d7ca"
PUBKEY="d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f"

ADDR=$($CMD getnewaddress)
TXID=$($CMD listunspent | jq -r .[0].txid)
UTX=$(RUST_LOG=off cargo run -- name new --privkey $PRIVKEY --json smith $TXID 0 $ADDR | jq -r .unsigned_tx)
STX=$($CMD signrawtransactionwithwallet $UTX | jq -r .hex)
$CMD sendrawtransaction $STX
$CMD generatetoaddress 3 $ADDR