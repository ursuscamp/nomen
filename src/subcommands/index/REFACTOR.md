# Refactoring steps

- [ ] Index raw_blockchain
- [ ] Scan raw_blockchain to parse properly indexable events
- [ ] Implement upgrade from v0
- [ ] Implement transfer from one to another
- [ ] Change to use tracing and tracing::subscriber instead of log
- [ ] Refactor commands
- [ ] Add API methods

# Things to test after refactor

- [ ] Test that block unwinding still works