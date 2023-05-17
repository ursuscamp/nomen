# Setting up a dev environment

The scripts in this folder can be used to set up a dev environment using regtest. These scripts are all expected to be run from the project root.

They should be run in the order documented here.

## Signet

`$ scripts/signet.sh`

This will start a signet bitcoind instance.

## Regtest (main testing environment)

`$ scripts/regtest.sh`

This script will startup a local regtest bitcoind instance for testing.

`$ scripts/create.sh`

This will generate and mine a block with a new name transaction. The name is `smith`.

`$ scripts/create_bad.sh`

This will generate a block with a second `smith` registration, which should be ignored by the indexer.

`$ scripts/records.sh`

This will create records for `smith`.

`$scripts/transfer.sh`

This will transfer `smith` to a new keypair.