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
- [ ] Resolve code TODOs
- [x] Remove unnecessary/unused code
- [ ] Version indicator on name page
- [ ] Add upgrade information
- [ ] Update docs
- [ ] Bump all versions in crates

# Things to test after refactor

- [ ] Test that block unwinding still works
- [ ] Test v0 create
- [ ] Test v0 -> v1 upgrade
- [x] Test v1 create
- [ ] Test v1 transfer
- [ ] Test v0 -> v1 upgrade -> v1 transfer
- [ ] Test duplicate names
- [ ] Test invalid transfer signature
- [x] Test name record

# Bugs

- [x] Continuously re-indexing from 112
- [ ] Uncorroborated claims page not working
- [ ] Feedback when event is broadcast