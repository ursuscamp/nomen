# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2023-11-24

### Bug Fixes

- do not redownload last event every index

- Re-download record events after reindex

- Better relay handling

- publis command should not use queue


### Features

- Updated config file format for index publishing.

- Validate config file on startup.

- NOM-04 support, relay publishing + .well-known

- Version subcomand #18

- record v1 upgrade block info

- UI will now warn users when attempting a transfer on a name that doesn't exist or shouldn't be transferred

- Added relays key to .well-known/nomen.json, per NOM-04 addition.

- "rebroadcast" command will rebroadcast known record events

- publish command to publish full relay index


### Other

- Preparing release v0.4.0

### Refactor

- Refactor: Refactored db module to sub-modules (#25)

- Refactor: Some tweaks to db submodule refactoring

- Refactor: Additional factoring on the db module


### Testing

- Test: Rewrote and refactored tests to base them on official test vectors (#24)



