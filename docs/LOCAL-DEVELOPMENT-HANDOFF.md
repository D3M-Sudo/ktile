# K-tile local development handoff

**Purpose:** resume development with a local coding agent on the existing `k-tile-rust-rebuild` branch, using the target Linux Mint 22.3 Cinnamon/Muffin host only where real desktop evidence is required.

**Repository:** `D3M-Sudo/ktile`  
**Branch:** `k-tile-rust-rebuild`  
**Baseline commit when this handoff was written:** `0adbfe488702811255d4fbba06a084b18a663111`  
**Primary target:** Linux Mint 22.3, Cinnamon/Muffin, X11  
**Language/toolchain:** Rust 1.85+ (edition 2024), Cargo  
**Current maturity:** early development; read-only with respect to live window geometry.

> This document is an execution queue, not a claim that the listed milestones are complete. Update statuses only after recording commands, results and the commit SHA tested.

---

## 1. Rules for the local agent

1. Start by inspecting `git status`, the current branch and HEAD. Work only on `k-tile-rust-rebuild`; do not modify `main`, `development` or `testing`.
2. Pull/fetch the latest branch before editing. The expected starting point is the baseline SHA above, but GitHub may have advanced since this file was written. Do not reset or force-push.
3. Read `AGENTS.md`, `README.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md` and `docs/HOST-VALIDATION-JOBS.md` before implementation.
4. Make small, reviewable commits with regression tests and update the roadmap/handoff as evidence changes.
5. Run formatting, all tests, Clippy, release build and license checks. Report exact commands and outcomes. Never infer test success from a green older CI run.
6. Do not install system packages, add dependencies, change desktop settings, enable autostart/services, or perform destructive desktop actions without asking first.
7. Do not open a PR, merge, rewrite history, force-push, or push future changes unless the repository owner explicitly authorizes that action. This handoff itself was committed directly to the existing feature branch at the owner's request.
8. Preserve the safety invariants below. In particular, **do not enable live geometry writes** during the first local development passes.

## 2. Current implementation boundary

The repository currently contains:

- backend-independent window/domain types and eligibility filtering;
- deterministic vertical-left/right and horizontal-top/bottom master/stack, maximized and grid planning;
- TOML configuration parsing/validation and CLI configuration/preview commands;
- read-only X11/EWMH inspection of clients, states, geometry, workspaces and work areas;
- RandR 1.5 monitor enumeration, with an explicitly diagnosed root-screen fallback when unavailable;
- a read-only event watcher with bounded event draining/coalescing and cooperative SIGINT/SIGTERM shutdown;
- a pure reconciliation planner that proposes Apply/Restore actions and tracks original/last-confirmed geometry;
- Xvfb/Openbox integration coverage which checks that preview commands do not mutate geometry;
- GitHub Actions quality/security workflows and dependency-license metadata checks.

Not implemented / not production-ready:

- a stateful event-driven daemon that drives the reconciler through confirmed backend operations;
- live geometry application and safe restoration;
- full layout parity, window insertion/removal/reordering semantics and persisted proportions;
- workspace/monitor-scoped state, rules, shortcuts, focus navigation, swaps and drag interactions;
- daemon/IPC, GTK4/Libadwaita UI, tray/status integration;
- user service/autostart, packaging and release artifacts;
- real Cinnamon/Muffin host validation.

The current `watch-plan` is a read-only preview refresher. K-tile must not be described or used as an automatic tiling manager yet.

## 3. First local session — establish a trustworthy baseline

Perform these steps in order before changing code.

### 3.1 Sync and inspect

```sh
git fetch origin
git switch k-tile-rust-rebuild
git pull --ff-only origin k-tile-rust-rebuild
git status --short --branch
git log -5 --oneline --decorate
git rev-parse HEAD
```

- Confirm the worktree is clean before starting. If it is not clean, inspect and preserve local changes; do not discard them.
- Confirm the branch is exactly `k-tile-rust-rebuild`.
- Record the actual HEAD SHA in the development notes.

### 3.2 Toolchain and host inventory

```sh
rustc --version
cargo --version
rustfmt --version
cargo clippy --version
uname -a
printf 'XDG_SESSION_TYPE=%s\nXDG_CURRENT_DESKTOP=%s\nDESKTOP_SESSION=%s\n' \
  "$XDG_SESSION_TYPE" "$XDG_CURRENT_DESKTOP" "$DESKTOP_SESSION"
```

Confirm the session is Cinnamon on X11 before running X11 host checks. Do not assume a Wayland session is supported.

### 3.3 Run the complete repository quality gate

From the repository root:

```sh
cargo fmt --all -- --check
cargo test --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release --all-features --locked
python3 scripts/check_dependency_licenses.py
```

Or run `make check` if it invokes the same checks. Capture full output and exit codes. The most recent layout fix is commit `0adbfe488702811255d4fbba06a084b18a663111`; specifically confirm the new regression test `master_stack_handles_large_cross_axis_without_u32_overflow` is discovered and passes.

If a check fails:
- fix the underlying issue; do not weaken lint/test gates or remove coverage;
- add a regression test where applicable;
- rerun the failed check and then the complete gate;
- do not proceed to host validation with a broken build.

### 3.4 Run the isolated X11 integration suite

Check whether these tools are already installed:

```sh
command -v Xvfb
command -v openbox
command -v xwininfo
command -v xdotool
```

If any are missing, stop and ask before installing packages. If available:

```sh
bash scripts/test-x11-integration.sh
```

The test must use its disposable X server/window manager and verify that `plan-live` does not change client geometry. Do not run this script against the active Cinnamon display unless its documentation explicitly says it is safe to do so.

### 3.5 Record baseline evidence

Record:
- tested commit SHA;
- Rust/Cargo versions;
- exact commands and pass/fail outcomes;
- whether the Xvfb integration test ran or was skipped and why;
- CI run URL/status for this exact commit, once GitHub Actions finishes.

Do not mark a milestone complete solely because the build succeeds.

---

## 4. Development sequence — repository work to complete locally

Complete these work packages in order. Each package should be a focused implementation/test/documentation change. Keep the desktop geometry backend read-only until the safety gate in section 6 is explicitly approved.

### REPO-01 — Finish layout semantics and parity

**Priority: first implementation task.**

1. Read `docs/CORTILE-PORTING.md` and inspect the referenced upstream Cortile behavior. Treat Cortile as a behavior reference; do not mechanically translate Go.
2. Add documented parity fixtures for vertical/horizontal master-stack, master count, master promotion/order, maximized and grid behavior.
3. Define deterministic insertion, removal and reordering behavior for every layout. Specify what happens when the focused/master window disappears and when the eligible window count crosses the master count.
4. Define gap and rounding invariants for odd dimensions, tiny work areas, large dimensions, negative monitor origins and multiple masters.
5. Review the just-fixed 64-bit partition arithmetic in `src/layout.rs`; retain the regression test and add edge/property tests where useful.
6. Add tests for stable output, no unintended overlap, positive dimensions, bounds and deterministic ordering. Maximized overlap is intentional and should be asserted separately.
7. Document any intentional divergence from Cortile and preserve MIT notices if source is copied/adapted.

**Acceptance:** fixtures and property tests cover insertion/removal/reordering and all layout variants; formatting, tests and Clippy pass. Update M1 in `docs/ROADMAP.md) only with evidence.

### REPO-02 — Complete safe reconciliation design, still disabled by default

1. Inspect `src/reconcile.rs`, `src/x11.rs` and the existing watcher/event model before changing interfaces.
2. Introduce a backend operation abstraction so the reconciler can be tested against a fake backend without a desktop.
3. Implement the event-driven stateful loop that consumes snapshots/events, computes plans and invokes the backend abstraction. Coalesce bursts and avoid polling/busy loops.
4. Immediately before **each** proposed mutation, re-read and validate: client still exists; it is still eligible; current state is not modal/fullscreen/maximized/transient/excluded; target monitor/work area is current; the geometry has not changed externally.
5. Record an applied geometry only after the backend confirms the server-side operation. Handle BadWindow/disappearing clients as isolated failures.
6. Keep a global and per-session explicit write gate disabled by default. A preview/dry-run mode must remain available and must never mutate geometry.
7. Implement restoration only when current geometry still equals the last geometry confirmed by K-tile. Never overwrite newer user/window-manager geometry.
8. Define shutdown behavior: stop accepting new work, drain/stop event handling safely, restore only eligible unchanged windows when explicitly enabled, then close backend resources.
9. Add fake-backend tests for stale snapshots, disappearing windows, external user moves, rejected requests, partial failures, duplicate events, shutdown and restoration conflict.
10. Add real child-process/Xvfb tests only after the abstraction is stable. The test must demonstrate that no write occurs when the gate is off.

**Acceptance:** safe fake-backend and integration tests pass; no geometry writes are possible by default; docs describe emergency disable and failure behavior. This does **not** authorize enabling writes on Cinnamon.

### REPO-03 — Workspace/monitor state, persistence and rules

1. Define stable identifiers for workspace and monitor scope; avoid depending on transient enumeration order alone.
2. Model per-workspace/per-monitor layout kind, master ratio/count, gaps and enable/disable state.
3. Add explicit behavior for sticky windows, floating windows and user exclusions. Keep special windows excluded by default.
4. Add rule matching for WM_CLASS and title with deterministic precedence, validation and diagnostics. Avoid arbitrary code execution or shell commands in rules.
5. Persist configuration/state under XDG paths. Use safe atomic writes (temporary file, fsync where appropriate, rename), preserve the previous valid file on failure and validate before activation.
6. Define schema versioning and migration behavior. Unknown fields/versions must fail clearly or be safely preserved according to the chosen format.
7. Test corrupt/truncated files, permissions, concurrent writes, missing directories, migrations, multiple monitors/workspaces and negative monitor coordinates.

**Acceptance:** pure tests cover state selection, rule precedence, persistence and migrations; no real desktop dependency is needed for the core model. Update configuration examples and docs.

### REPO-04 — Interaction model and shortcut safety

1. Define backend-independent commands for focus next/previous, promote focused window to master, swap windows, cycle layout and enable/disable/restore.
2. Specify deterministic focus ordering and behavior when the focused client disappears or is excluded.
3. Add shortcut configuration parsing, key normalization and duplicate/conflict detection.
4. Never silently overwrite Cinnamon keybindings. Provide diagnostics and a manual setup path; desktop registration must be opt-in and reversible.
5. Implement operations behind backend traits and test with fakes. Keep drag interactions separate from the core layout engine.
6. Document which operations require Muffin/X11 support and which are pure domain behavior.

**Acceptance:** unit tests cover navigation, promotion, swaps, invalid keys, duplicate bindings and conflict reporting. No global shortcut is registered during tests.

### REPO-05 — Daemon, IPC and optional UI

1. Split long-running daemon lifecycle from CLI diagnostics. Daemon must own the X11 connection and stateful reconciler.
2. Use session-scoped local IPC; prefer D-Bus or another authenticated/session-local mechanism. Do not expose an unauthenticated TCP listener.
3. Define a versioned request/response protocol, bounded messages, timeouts, clear errors and authorization boundaries. GUI/CLI clients must not bypass the daemon's safety gate.
4. Ensure only one daemon instance per user session; handle stale sockets/locks and clean shutdown.
5. Keep the core and daemon headless-testable. Add GTK4/Libadwaita only as an optional feature and only after checking CI build dependencies and dependency-license impact.
6. UI must expose status, diagnostics, layout/workspace/monitor settings and explicit enable/disable/restore controls. It must never enable geometry writes silently.
7. Test IPC protocol, invalid/oversized messages, client disconnects, daemon restart, stale socket recovery and permission boundaries.

**Acceptance:** daemon and IPC tests run in CI without Cinnamon; GUI is optional and does not become a mandatory dependency for headless builds.

### REPO-06 — Packaging and release engineering

1. Decide packaging targets based on the supported Linux Mint/Cinnamon session and document trade-offs before adding manifests.
2. Add user-scoped systemd service and desktop autostart files only after daemon lifecycle is stable. Do not enable them automatically on install.
3. Provide install/uninstall scripts or package metadata with safe preservation of user configuration and explicit service enable/disable steps.
4. Add architecture CI for supported targets (at minimum x86_64; add aarch64 only when dependencies/builds are validated).
5. Keep formatting, tests, Clippy, release build, dependency audit and license inventory as blocking CI jobs.
6. Generate release artifacts reproducibly; record toolchain, target triples, build commands, checksums and dependency/license notices.
7. Verify the final installed tree contains all required license/attribution files, including any adapted Cortile source notices.
8. Add upgrade, rollback, uninstall and config migration checks.

**Acceptance:** clean build/install/uninstall in disposable environments; CI covers declared architectures; release artifacts and attribution inventory are reproducible. Do not publish a release without explicit approval.

---

## 5. Target-host validation — run on Linux Mint 22.3 Cinnamon/X11

These jobs are deliberately separate from repository implementation. Run them in order and record the tested commit and environment in `docs/HOST-VALIDATION-JOBS.md`.

### HOST-01 — Read-only Cinnamon/Muffin baseline

After section 3 passes:

```sh
cargo build --release --all-features --locked
target/release/k-tile check-x11
target/release/k-tile inspect-windows
RUST_LOG=debug target/release/k-tile watch-x11 > /tmp/ktile-host-watch.log 2>&1 &
watcher_pid=$!
sleep 2
kill -TERM "$watcher_pid"
wait "$watcher_pid"
grep -F "X11 watcher received cooperative shutdown request" /tmp/ktile-host-watch.log
```

Confirm Cinnamon remains responsive and that focus, stacking, decorations, panels and workspace switching are unchanged. Watcher must exit cleanly and emit the shutdown marker. These commands are read-only; do not run as root.

### HOST-02 — Window state/lifecycle matrix

Test normal, maximized, fullscreen, modal, transient/dialog, floating, sticky, minimized and disappearing windows. Capture `inspect-windows` and `plan-live` output before/after state changes. Run `watch-x11`/`watch-plan) while opening and closing clients.

Acceptance: documented special windows are excluded; disappearing clients do not crash the process; previews refresh on relevant events; client geometry remains unchanged.

### HOST-03 — Workareas, workspaces and monitor topology

Test panels/docks, workspace switching, monitor attach/detach if available, and monitors positioned left/above the primary display (negative X/Y origins). Capture `check-x11`, `plan-live) and watcher logs at each transition.

Acceptance: no guessed geometry, correct workarea/monitor mapping, events observed and no stale preview after topology changes.

### HOST-04 — Idle/active resource baseline

While the watcher is idle, record CPU and RSS after 1, 5 and 15 minutes; repeat during window open/close, workspace changes and monitor changes:

```sh
ps -o pid,pcpu,rss,stat,etime,cmd -p <PID>
```

Record machine load and sampling method. Acceptance: no busy loop, unbounded RSS growth or sustained idle CPU activity.

### HOST-05 — Live-write safety gate (blocked)

**Do not execute this job until REPO-02 is implemented, independently reviewed, and the owner explicitly authorizes the test.**

Required before any live mutation:
- opt-in gate disabled by default and visible in config/UI;
- immediate pre-request revalidation of client, eligibility, geometry and work area;
- backend-confirmed operations only;
- original geometry captured and restore guarded against external changes;
- fake-backend and real child-process integration tests;
- documented emergency disable/restore procedure;
- disposable test session and explicit owner approval.

Start with a disposable X11 session, not the daily desktop. Do not enable autostart.

### HOST-06 — Packaging/session lifecycle

Only after REPO-05/06 are ready. Test user service start/stop/restart, login/logout, single-instance behavior, stale IPC cleanup, uninstall and preservation of user configuration. Inspect service files before enabling them.

---

## 6. Non-negotiable safety invariants

- No live geometry mutation until REPO-02 is implemented/tested and HOST-01 through HOST-04 pass.
- HOST-05 additionally requires explicit owner approval; this handoff is not that approval.
- Never restore over geometry changed by the user or window manager.
- Revalidate immediately before every mutation, not merely once per event batch.
- Never assume a fixed resolution, zero-origin monitor, stable monitor enumeration order or available EWMH metadata.
- Exclude modal, fullscreen, maximized, transient, panel/dock, desktop and user-excluded windows by default.
- Handle disappearing clients and backend errors without crashing the daemon.
- Event processing must block while idle and use bounded coalescing.
- Do not auto-start, install a user service or publish a release during development.
- Preserve MIT and upstream notices. Audit dependency licenses whenever dependencies change.

## 7. CI and completion policy

The owner previously verified that the branch's CI jobs were green after the workflow fixes. That result applies only to the commit tested at that time. The layout-fix commit and this handoff commit must each receive their own CI result before being treated as validated.

For every development package, record:

| Evidence | Required record |
|---|---|
| Source | commit SHA and changed files |
| Formatting | exact command and result |
| Tests | exact command, count and result |
| Clippy | exact command and result |
| Release build | exact command and result |
| X11 integration | run/skip reason and result |
| CI | run URL, commit SHA and final status |
| Host validation | OS/session/WM versions, steps, logs and result |
| Roadmap | acceptance criteria met and evidence links |

Do not mark M0–M8 or REPO-01–06 complete without their acceptance evidence. Keep the README status accurate: K-tile remains read-only and early-development until safe live tiling and target-desktop validation are actually complete.

## 8. Suggested first task for the local agent

Start with **section 3 (baseline)**, then work only on **REPO-01**. Do not begin live geometry writes. Once REPO-01 has complete fixtures and passing tests, update the roadmap and propose the next focused REPO-02 design/implementation slice. Keep HOST-01–04 queued for the owner to run on the real Cinnamon/Muffin machine after the repository build and Xvfb suite pass.
