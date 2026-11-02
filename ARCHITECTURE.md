# K-tile architecture

The maintained architecture specification is [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). It describes the intended process boundaries, safety invariants, backend contracts and the current implementation boundary.

## Current implementation boundary

The repository currently contains a Rust domain model, deterministic layout planner, TOML configuration, CLI previews, read-only X11/EWMH discovery, a diagnostic event watcher and a pure reconciliation planner. It does **not** yet change live window geometry, run an automatic tiling daemon, provide the GTK4/Libadwaita GUI or package desktop integration. Do not treat the branch as production-ready.

See [docs/ROADMAP.md](docs/ROADMAP.md) for verified milestone status and [docs/TESTING.md](docs/TESTING.md) for automated and manual validation requirements.
