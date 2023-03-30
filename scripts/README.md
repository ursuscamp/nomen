# Setting up a dev environment

The scripts in this folder can be used to set up a dev environment using regtest. These scripts are all expected to be run from the project root.

They should be run in the order documented here.

`$ scripts/regtest.sh`

This script will startup a local regtest bitcoind instance for testing.

`$ scripts/create.sh`

This will generate and mine a block with a new name transaction. The name is `smith` and the children are `bob.smith` and `alice.smith`.

`$ scripts/update.sh`

This will generate and mine a block with an update to the `smith` name, adding `greg.smith` to it.