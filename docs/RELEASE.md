# Preparing A Release

1. Crate release branch.
2. Archive current `CHANGELOG.md` to `docs/changelogs/vY.Y.Y.md`, replacing `Y.Y.Y` with the current version number.
3. Archive current `RELEASE_NOTES.md` to `docs/release_notes/vY.Y.Y.md`, replacing `Y.Y.Y` with the current version number.
4. Update version numbers in `Cargo.toml` files to new version.
5. Run `git cliff -o CHANGELOG.md --unreleased --tag vX.X.X`, replacing `X.X.X` with the new version.
6. Write new `RELEASE_NOTES.md`, containing highlights of the release, and special release instructions that must be done prior to upgrade.
7. Commit with a `release:` tag on the commit message.
8. Open PR to `master` and merge.
9. `git tag vY.Y.Y && git push --tags` to create the release and build the artifacts.