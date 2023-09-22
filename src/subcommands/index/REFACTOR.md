# Refactoring steps

- [x] Index raw_blockchain
- [x] Scan raw_blockchain to parse properly indexable events
- [x] Implement upgrade from v0
  - [x] V1 Create should automatically update any older matching V0 create
- [ ] Implement transfer from one to another
- [ ] Change to use tracing and tracing::subscriber instead of log
- [ ] Refactor commands
  - [ ] Move CLI into seperate binary?
- [ ] Add API methods
  - [ ] Generating PSBT
  - [ ] Filling in PSBT

# Things to test after refactor

- [ ] Test that block unwinding still works
- [ ] Test v0 create
- [ ] Test v0 -> v1 upgrade
- [ ] Test v1 create
- [ ] Test v1 transfer
- [ ] Test v0 -> v1 upgrade -> v1 transfer
- [ ] Test duplicate names