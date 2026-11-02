# K-tile development roadmap

Status labels describe verified repository implementation, not aspiration. A milestone is complete only when its acceptance tests and documentation are present.

## M0 — Rust foundation (in progress)

- [x] Rust crate and modular domain model.
- [x] Deterministic initial layout planner.
- [x] TOML config schema and validation.
- [x] Read-only X11/EWMH connection probe.
- [x] Unit tests for layout and eligibility policy.
- [x] Run Cargo quality gates in GitHub Actions: formatting, tests, Clippy and release build pass on the current development commit.
- [ ] Confirm exact Cortile source files and record per-file attribution before any code is ported.

## M1 — Layout parity and geometry hardening

- [ ] Port and test Cortile's vertical/horizontal layout behavior, proportion persistence, and master/stack rules.
- [ ] Define rounding and gap invariants, including tiny work areas and negative monitor origins.
- [x] Property tests for non-overlap, bounds and positive dimensions.
- [ ] Idempotence and window insertion/removal/reordering semantics.
- [ ] Add window insertion/removal/reordering semantics.

## M2 — X11 read-only inventory

- [x] EWMH capability detection and client-list discovery.
- [x] Read-only EWMH metadata: title, WM_CLASS, state, root-relative client geometry, desktop count/current desktop and work area.
- [x] RandR 1.5 active-monitor enumeration and EWMH work-area reads; no guessed display geometry.
- [ ] Validated fallback for servers without RandR 1.5; currently the CLI reports an explicit diagnostic.
- [x] Read-only diagnostic event watcher for basic X11 window lifecycle events.
- [ ] Xvfb/Openbox integration tests for supported behavior.
- [x] No geometry writes are issued in the read-only inventory milestone.

## M3 — Event-driven reconciler and safe application

- [x] Pure reconciliation planner proposes Apply/Restore actions without X11 I/O.
- [x] Capture original geometry and track last confirmed K-tile geometry; suppress windows after external geometry changes.
- [ ] Event loop for map/unmap/configure/property/focus/workspace/monitor events.
- [ ] Coalesced reconciliation with bounded work and no busy loop.
- [ ] Re-read eligibility state immediately before each geometry request.
- [ ] Opt-in geometry write gate until target desktop verification.
- [ ] Cooperative shutdown and safe restoration tests using a real child process.

## M4 — Workspaces, monitors and rules

- [ ] Per-workspace layout state and proportion persistence.
- [ ] Multi-monitor support including negative coordinates and monitor hotplug.
- [ ] Class/title rules, floating/sticky behavior and configurable exclusions.
- [ ] Panels, struts and usable workarea support.

## M5 — Interaction

- [ ] Configurable keyboard shortcuts without silently overriding Cinnamon shortcuts.
- [ ] Focus navigation and master promotion.
- [ ] Drag-and-drop window swap.
- [ ] Tray/status menu and explicit enable/disable/restore controls.
- [ ] Respect the existing window manager's behavior and provide conflict diagnostics.

## M6 — GTK4/Libadwaita GUI and IPC

- [ ] GTK4/Libadwaita settings and status UI.
- [ ] Session-scoped IPC between GUI and daemon.
- [ ] TOML editing with validation, diagnostics and safe atomic writes.
- [ ] UI controls for layouts, workspace/monitor state, rules and shortcuts.
- [ ] GUI optional for headless/test deployments.

## M7 — Cinnamon/Muffin validation

- [ ] Test on Linux Mint 22.3 Cinnamon X11.
- [ ] Test modal, fullscreen, maximized, floating, sticky, transient and disappearing windows.
- [ ] Test panel/workarea changes, workspace switching and multi-monitor hotplug.
- [ ] Measure idle/active CPU and resident memory.
- [ ] Verify no regression to Muffin focus, decorations or workspace behavior.

## M8 — Packaging and release readiness

- [ ] User systemd service and desktop autostart integration.
- [ ] Install/uninstall and config migration docs.
- [ ] License and dependency inventory, including copied Cortile portions.
- [ ] CI for formatting, tests, Clippy, security/dependency review and supported architectures.
- [ ] Release artifacts and reproducible build notes.

## Definition of done

No milestone is marked complete solely because the project compiles. Required evidence includes tests, explicit limitations, clean shutdown behavior where relevant, and target-desktop tests for any live window management changes.
