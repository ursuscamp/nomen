#!/usr/bin/env bash

mkdir -p .bitcoin
bitcoind -regtest -datadir=.bitcoin -rpcuser=regtest -rpcpassword=regtest -txindex