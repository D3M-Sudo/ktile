# K-tile roadmap

The current, maintained roadmap is [docs/ROADMAP.md](docs/ROADMAP.md). That document is the source of truth for milestone status and acceptance criteria.

## Current state

- **Implemented:** Rust crate, domain model, deterministic layout planning, TOML validation, CLI previews, read-only X11/EWMH inspection, reconciliation planning, unit/property tests, CI quality gates, dependency advisory audit, dependency-license metadata check and an isolated Xvfb/Openbox smoke test.
- **Not implemented yet:** live geometry mutation, event-driven tiling daemon, verified restoration, workspace/monitor orchestration, rules, shortcuts, drag-and-drop, tray, GTK4/Libadwaita GUI, packaging and real Cinnamon/Muffin validation.
- **Release status:** not production-ready; do not enable desktop autostart.

Milestones are not considered complete based only on compilation. See [docs/TESTING.md](docs/TESTING.md) and [docs/RELEASING.md](docs/RELEASING.md) for required evidence.
