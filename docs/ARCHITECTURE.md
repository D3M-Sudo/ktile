# K-tile architecture

## Goals

K-tile is a lightweight Rust tiling manager that cooperates with an existing window manager. Initial target: Cinnamon 22.3 / Muffin / X11. Wayland support is explicitly future work because a generic client-side process cannot assume compositor control equivalent to X11 EWMH.

## Process boundaries

1. **Domain/core** — window metadata, eligibility policy, geometry, layouts and deterministic plans. No X11 or GTK dependencies are required by the domain model.
2. **X11 adapter** — EWMH/ICCCM properties, root/client discovery, RandR monitors, event subscriptions and validated geometry requests.
3. **Reconciler** — turns backend snapshots and user configuration into plans; compares desired and actual geometry; avoids redundant requests and handles stale windows.
4. **Daemon** — owns the backend connection and event loop. Shutdown is cooperative and restores captured geometry only where safe.
5. **IPC** — local control surface for GUI, CLI and desktop integration. Prefer an authenticated/session-scoped D-Bus service rather than an unauthenticated network listener.
6. **GTK4/Libadwaita GUI** — configuration editing, live status and explicit enable/disable controls. It must not be required for the daemon's steady-state operation.
7. **Desktop integration** — user systemd service, autostart, tray/status indicator, shortcuts and packaging.

## Invariants

- Planning is deterministic for identical input.
- The layout engine cannot issue window-system requests.
- Special windows are excluded by default.
- Configuration is validated before changing desktop state.
- Live geometry writes are opt-in until Cinnamon/Muffin target tests pass.
- Revalidate a window immediately before each geometry change.
- Never assume a monitor is 1920x1080; use validated RandR/root geometry or fail safely.
- Never overwrite the user's newer geometry when restoring.
- Backend failures for a disappearing window are isolated and do not crash the whole daemon.
- No busy loop; event-driven reconciliation with bounded coalescing.
- All persistent paths respect XDG base-directory conventions.
- GUI and backend are independently testable.

## Layout model

Supported initial planner kinds: vertical master-left/right, horizontal master-top/bottom, maximized single-window layout and grid. The master count and master area ratio are configuration-controlled. Floating and special-state windows are omitted from plans rather than receiving accidental geometry.

All geometry is represented in signed X/Y coordinates and unsigned positive dimensions. The planner rejects invalid work areas and invalid master ratios. Multi-monitor coordinate origins can be negative; geometry calculations must not assume a zero origin.

## Backend abstraction

The future backend contract should expose:
- connection/session lifecycle;
- enumeration and snapshots of managed windows;
- monitor/workarea enumeration through RandR/EWMH, with explicit failure when required metadata is unavailable;
- event stream for map, unmap, configure, focus, state, workspace and monitor changes;
- explicit operations for focus, move-to-workspace, and geometry application;
- revalidation before mutation.

A future Wayland backend must be compositor-specific where necessary and must not claim support merely because a protocol library compiles.

## GUI and resource use

GTK4/Libadwaita is feature-gated during development so headless CI can test core logic without pulling the GUI stack. The daemon remains separate from the GUI. Avoid background polling and redundant geometry writes.

## Current implementation boundary

The current code includes a pure layout planner, validated TOML configuration, a read-only X11/EWMH inventory, a live-window layout preview that emits proposed geometry without applying it, and a pure reconciliation planner that emits Apply/Restore proposals. The reconciler captures original geometry, requires explicit backend confirmation before recording applied geometry, and suppresses windows whose geometry changed outside K-tile. The diagnostic event watcher is read-only and coalesces client lifecycle/geometry, focus, EWMH workspace/state, and RandR topology notifications. `watch-plan` refreshes a read-only preview on those events; it does not yet drive the stateful `Reconciler` through confirmed backend operations. K-tile does not mutate window geometry. Cooperative signal-driven shutdown and end-to-end monitor-hotplug validation remain open.
