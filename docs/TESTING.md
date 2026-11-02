# Testing strategy

## Automated quality gates
Run from the repository root:

    make check

Or run the individual commands:

    cargo fmt --all -- --check
    cargo test --all-features --locked
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo build --release --all-features --locked
    python3 scripts/check_dependency_licenses.py

CI must treat each check as blocking. A failed formatter, test or Clippy run must not be downgraded to a warning.

## Test layers
1. Pure core tests: eligibility, config validation, deterministic layout output, bounds, tiny dimensions and negative monitor origins.
2. Property tests: generated areas/window counts; positive rectangle dimensions, bounds, no unintended overlaps, and deterministic output.
3. X11 integration: disposable X server such as Xvfb with an EWMH-aware test WM; create/destroy clients and validate metadata/events. Exercise bursty event delivery and verify the watcher remains idle-blocking, processes no more than 128 queued events per batch, and `watch-plan` refreshes after a relevant event without geometry mutation.
4. Lifecycle: real child process shutdown, bounded termination and safe restoration without overwriting newer user geometry.
5. Target desktop: Linux Mint 22.3 Cinnamon/Muffin X11; panels/workareas, fullscreen, modal/transient, workspace changes and monitor hotplug.
6. Resource checks: idle/active CPU and RSS; no busy loops or repeated identical geometry requests.

Run the isolated smoke test locally after building the release binary and installing Xvfb, Openbox, x11-utils and xdotool:

    cargo build --release --all-features
    bash scripts/test-x11-integration.sh

## Manual Cinnamon/Muffin checklist
- [ ] Start and stop without disrupting Muffin or Cinnamon.
- [ ] Confirm decorations, focus, stacking and shortcuts.
- [ ] Verify panels/docks and workareas are respected.
- [ ] Test fullscreen, maximized, modal, transient, floating, sticky and minimized windows.
- [ ] Switch workspaces while windows open and close.
- [ ] Attach/detach monitors, including negative-coordinate layouts.
- [ ] Destroy a test client during reconciliation.
- [ ] Change geometry manually, then stop the app; restoration must not overwrite newer user actions.
- [ ] Measure idle CPU and memory.

Record OS, Cinnamon/Muffin versions, monitor setup and results. CI does not replace target-desktop tests.
