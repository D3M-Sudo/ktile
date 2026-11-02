# Host validation and owner-input job queue

This file is the handoff ledger for work that cannot be completed or truthfully verified from GitHub-hosted CI alone. Update each job with the tested commit, environment, evidence, date and result. Do not mark a job complete from a successful compile or Xvfb run.

## How to report a completed job

For each job, provide:
- K-tile commit SHA tested;
- Linux Mint, Cinnamon, Muffin and kernel versions;
- X11 session details and monitor topology (resolution, scale, arrangement, including negative-coordinate monitors);
- exact commands and steps performed;
- logs or screenshots for failures and unexpected behavior;
- CPU/RSS observations where requested.

Do not enable live geometry writes, autostart or a user service during the read-only validation jobs.

## Jobs requiring the user's local host

### HOST-01 — Cinnamon/Muffin compatibility baseline
- **Status:** queued — requires the target desktop.
- **Purpose:** verify the current read-only backend and watcher do not interfere with Cinnamon/Muffin.
- **Before starting:** build the branch and run only the diagnostic commands below.
- **Commands:**
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
- **Checks:** Cinnamon remains responsive; focus, stacking, decorations, panels and workspace switching remain unchanged; watcher exits with status 0 after SIGTERM and logs the cooperative-shutdown marker; no unexpected errors.
- **Safety:** all current commands are read-only. Do not run as root.

### HOST-02 — Window-state and lifecycle matrix
- **Status:** queued — requires real desktop applications/windows.
- **Purpose:** validate observed metadata and eligibility for normal, maximized, fullscreen, modal, transient/dialog, floating, sticky, minimized and disappearing windows.
- **Procedure:** open representative windows; capture `inspect-windows` output; run `plan-live`; change states and repeat. Close a window while `watch-x11` and `watch-plan` are active.
- **Acceptance:** special windows are excluded as documented; disappearing clients produce isolated diagnostics, not a process crash; preview refreshes on relevant state/workspace/focus events.
- **Safety:** preview only; verify geometry is unchanged.

### HOST-03 — Workareas, workspaces and monitor topology
- **Status:** queued — requires physical Cinnamon session and, for hotplug, monitor hardware.
- **Purpose:** validate EWMH workareas, workspace changes, RandR notifications and coordinate mapping.
- **Procedure:** test with panels/docks; switch workspaces; if available, attach/detach a monitor and test a layout with a monitor left/above the primary display (negative X/Y origin). Capture `check-x11`, `plan-live` and watcher logs before/after each transition.
- **Acceptance:** no guessed geometry; correct monitor/workarea identification; events observed; no stale preview after topology changes.
- **Safety:** no live geometry mutation.

### HOST-04 — Idle/active resource baseline
- **Status:** queued — requires target host.
- **Purpose:** measure daemon-candidate resource behavior before long-running integration.
- **Procedure:** while `watch-x11` is idle, record CPU and RSS after 1, 5 and 15 minutes; repeat during window open/close, workspace and monitor changes. Use `ps -o pid,pcpu,rss,stat,etime,cmd -p <PID>`.
- **Acceptance:** no busy loop, unbounded RSS growth or sustained CPU activity while idle. Record machine load and sampling method.

### HOST-05 — Live geometry write safety gate
- **Status:** blocked — do not run until a separately reviewed implementation enables writes.
- **Purpose:** validate per-request revalidation, geometry application, external-user-change suppression, safe restore and shutdown rollback.
- **Required evidence before execution:** explicit opt-in configuration; immediate re-read of window existence, eligibility and geometry before every request; only record confirmed server-side operations; never restore over geometry changed by the user/window manager; real child-process integration tests; documented emergency disable/restore path.
- **Acceptance:** test on a disposable user session first, then Cinnamon. This job requires explicit user approval before enabling or executing live writes.

### HOST-06 — Packaging and session lifecycle
- **Status:** queued — after daemon/IPC and install targets exist.
- **Purpose:** verify user-scoped service/autostart, startup ordering, restart, logout/login, uninstall and config preservation.
- **Acceptance:** no root daemon, no duplicate instances, no stale IPC/socket, no unexpected config deletion, clean stop on logout.
- **Safety:** defer until packaging and lifecycle implementation is ready; inspect service files before enabling.

## Jobs that can be completed in repository/CI without host access

These remain engineering work, not user-host jobs:
- **REPO-01:** finish deterministic layout parity and insertion/removal/reordering semantics with fixtures and property tests.
- **REPO-02:** implement stateful event-driven reconciliation behind a disabled-by-default geometry-write gate; add backend operation abstraction and stale-window revalidation.
- **REPO-03:** add per-workspace/per-monitor layout state, persistence and explicit rules model.
- **REPO-04:** implement interaction (focus navigation, master promotion, swaps and shortcut conflict diagnostics).
- **REPO-05:** implement daemon/IPC and optional GTK4/Libadwaita UI.
- **REPO-06:** implement packaging, dependency/license attribution audit, architecture matrix and reproducible release artifacts.

Repository jobs must not be marked done until implementation, tests and documentation are committed and CI is green. Keep live window mutation disabled until HOST-01 through HOST-04 pass and HOST-05 is explicitly authorized.
