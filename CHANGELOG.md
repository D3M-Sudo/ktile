# Changelog

K-tile has not yet published a stable release.

## Unreleased
- Established the Rust crate, domain model and deterministic layout planner.
- Added TOML configuration parsing and validation plus CLI commands for config initialization, validation and layout previews.
- Added a read-only X11/EWMH adapter with client geometry, RandR 1.5 monitor discovery and EWMH work-area reads, plus a diagnostic event watcher.
- Added a pure reconciliation planner with confirmed-application tracking and guarded restore proposals.
- Added a committed Cargo.lock, locked CI builds and a dependency advisory audit workflow.
- Added initial architecture, roadmap and Cortile attribution guidance.
- Added repository security, contribution, agent, usage and release documentation.
- Added a CI Xvfb/Openbox smoke test for EWMH discovery, diagnostic events and TOML-driven preview behavior.
- Added automated dependency license-metadata completeness checks and explicit documentation that they are not a license-compatibility verdict.
- Hardened layout coordinate arithmetic for large dimensions and explicitly rejects unrepresentable work areas or tiny areas that cannot fit a non-overlapping layout.
- Fixed layout previews to consume validated TOML layout-engine settings when a config file exists.
- Added `plan-live`, a read-only preview over current EWMH clients, grouped by discovered monitor and clipped to the active desktop work area; it never applies geometry changes.
- Added tests for negative-origin monitor/work-area intersection, disjoint monitor areas, XDG configuration path handling, and configuration validation boundaries.
- Applied the TOML `log_level` setting to tracing by default, while preserving `RUST_LOG` as the explicit override.
- Added a CI test/build path with default features disabled to verify the backend-independent core does not require X11.
- Hardened the default configuration path to ignore a relative `XDG_CONFIG_HOME` rather than writing configuration into the working directory.
- Automatic live tiling, GUI, tray, workspace/monitor management, hotkeys, drag interactions and packaging remain unfinished.
