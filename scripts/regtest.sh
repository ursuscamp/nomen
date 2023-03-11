#!/usr/bin/env bash
CREDS="-regtest -rpcuser=regtest -rpcpassword=regtest"

while getopts r flag
do
    case "${flag}" in
        r) rm -rf .bitcoin;;
    esac
done

mkdir -p .bitcoin
bitcoind -daemon -datadir=.bitcoin -regtest $CREDS -txindex
sleep 1.5
bitcoin-cli $CREDS createwallet regtest
ADDRESS=$(bitcoin-cli $CREDS -rpcwallet=regtest getnewaddress)
echo "$ADDRESS"
bitcoin-cli $CREDS -rpcwallet=regtest generatetoaddress 101 $ADDRESS
tail -f .bitcoin/regtest/debug.log