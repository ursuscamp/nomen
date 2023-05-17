#!/usr/bin/env bash
CREDS="-signet -rpcuser=signet -rpcpassword=signet"

while getopts r flag
do
    case "${flag}" in
        r) rm -rf .bitcoin;;
    esac
done

mkdir -p .bitcoin
bitcoind -daemon -datadir=.bitcoin -signet $CREDS -txindex
tail -f .bitcoin/signet/debug.log