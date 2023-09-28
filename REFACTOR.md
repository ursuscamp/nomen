# Refactoring steps

- [x] Index raw_blockchain
- [x] Scan raw_blockchain to parse properly indexable events
- [x] Implement upgrade from v0
  - [x] V1 Create should automatically update any older matching V0 create
- [x] Implement transfer from one to another
- [x] Change to use tracing and tracing::subscriber instead of log
- [x] Refactor commands
  - [x] Move CLI into seperate binary?
- [ ] Add API methods
  - [ ] Generating PSBT
  - [ ] Filling in PSBT
- [ ] Add a checkpoint system to rewind a hundred blocks back
  - [ ] Delete from stale blocks from `index_height`, `raw_blockchain`
  - [ ] Delete entire `blockchain_index`
  - [ ] Re-run blockchain index from latest
- [ ] Update SPEC
  - [x] Update SPEC text
  - [ ] Mark correct date on spec changes section.
- [ ] Fully clippy check
- [ ] Remove unnecessary/unused code
- [ ] Remove anyhow from nomen_core
- [ ] API docs

# Things to test after refactor

- [ ] Test that block unwinding still works
- [ ] Test v0 create
- [ ] Test v0 -> v1 upgrade
- [ ] Test v1 create
- [ ] Test v1 transfer
- [ ] Test v0 -> v1 upgrade -> v1 transfer
- [ ] Test duplicate names
- [ ] Test invalid transfer signature