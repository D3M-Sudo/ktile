# Release checklist

K-tile is not yet release-ready. Do not publish a stable release until applicable items have evidence.

## Pre-release
- [ ] Confirm scope, known limitations and version metadata.
- [ ] Pass formatting, tests, Clippy and release build from a clean checkout.
- [ ] Commit and review Cargo.lock for the application.
- [ ] Run X11 integration tests in a disposable session.
- [ ] Complete Linux Mint 22.3 Cinnamon/Muffin manual checks.
- [ ] Test shutdown, restore, user geometry changes, workspace switching and monitor hotplug.
- [ ] Audit direct/transitive dependency licenses and advisories.
- [ ] Preserve Cortile notices for each copied/adapted source file.
- [ ] Update README, usage, installation and changelog.
- [ ] Verify architecture/runtime dependencies and reproducible build instructions.

## Publishing
1. Create a versioned changelog entry.
2. Tag the reviewed commit.
3. Attach checksums and build provenance where available.
4. Publish release notes with tested platforms and explicit non-support statements.
5. Keep rollback instructions and a known-good prior artifact available.

Do not claim Wayland support until a compositor-specific backend and end-to-end tests exist.
