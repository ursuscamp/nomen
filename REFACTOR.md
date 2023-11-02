# Refactoring steps

- [x] Index raw_blockchain
- [x] Scan raw_blockchain to parse properly indexable events
- [x] Implement upgrade from v0
  - [x] V1 Create should automatically update any older matching V0 create
- [x] Implement transfer from one to another
- [x] Change to use tracing and tracing::subscriber instead of log
- [x] Refactor commands
  - [x] Move CLI into seperate binary?
  - [ ] Full set of CLI commands
- [x] Add a checkpoint system to rewind a hundred blocks back
  - [x] Delete entire `blockchain_index`
  - [x] Delete from stale blocks from `index_height`, `raw_blockchain`
  - [x] Re-run blockchain index from latest
- [ ] Update SPEC
  - [x] Update SPEC text
  - [ ] Mark correct date on spec changes section.
- [x] Fully clippy check
- [x] Remove anyhow from nomen_core
- [x] API docs
- [x] CORS headers for API
- [x] Resolve code TODOs
- [x] Remove unnecessary/unused code
- [x] Version indicator on name page
- [x] Add upgrade information
- [x] Update docs
- [x] Bump all versions in crates
- [x] Set 100 block expiration on transfer cache
- [x] Add transfer option in UI
- [x] Add re-index command.
- [ ] Add re-scan command.

# Things to test after refactor

- [x] Test that block unwinding still works
- [x] Test v0 create
- [x] Test v0 -> v1 upgrade
- [x] Test v1 create
- [x] Test v1 transfer
- [x] Test v0 -> v1 upgrade -> v1 transfer
- [x] Test duplicate names
- [ ] Test invalid transfer signature
- [x] Test name record
- [ ] test npubs

# Bugs

- [x] Continuously re-indexing from 112
- [x] Creating a v0 results in an empty name in list before there is a record broadcast
- [x] Feedback when event is broadcast
- [x] Update Records link isn't working from name page
- [x] Still loggin lots of transfer messages for some reason
  - [x] Create vw for unindexed blocks, including using the transfer cache
- [x] Item left in transfer cache after transfer is complete