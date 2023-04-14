#!/usr/bin/env bash
CREDS="-regtest -rpcuser=regtest -rpcpassword=regtest"
CMD="bitcoin-cli $CREDS -rpcwallet=regtest"

PRIVKEY="c5f6081f125bb3d086c1c5a030d3187051888887053d4c99b88f4e168c516ce6"
PUBKEY="8f1024f11eb4ef98e3f34aeb07d518c99b3b9fa4abf5900d2b3446c8b9987883"

ADDR=$($CMD getnewaddress)
TXID=$($CMD listunspent | jq -r .[0].txid)
UTX=$(RUST_LOG=off cargo run -- name new --privkey $PRIVKEY smith $TXID 0 $ADDR | grep "Unsigned Tx" | awk '{print $3}')
STX=$($CMD signrawtransactionwithwallet $UTX | jq -r .hex)
$CMD sendrawtransaction $STX
$CMD generatetoaddress 3 $ADDR