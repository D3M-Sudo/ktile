# K-tile documentation

Documentation is grouped by purpose. Start with the section matching the task.

## Project and specifications
- [Project overview](project/OVERVIEW.md) — purpose, target platform and design goals.
- [Functional specification](specifications/FUNCTIONAL-SPECIFICATION.md) — expected behavior and layout semantics.
- [Roadmap](planning/ROADMAP.md) — verified milestone status and acceptance criteria.
- [Original M0–M8 plan](planning/MILESTONES-M0-M8.md) — phased plan and historical constraints.

## Architecture and platform
- [Architecture](architecture/ARCHITECTURE.md) — component boundaries, invariants and implementation status.
- [Rust/x11rb stack](architecture/RUST-X11RB-STACK.md) — technology and module-boundary decisions.
- [EWMH/Muffin compatibility](architecture/EWMH-MUFFIN-COMPATIBILITY.md) — target desktop behavior and validation.

## Development
- [Local development handoff](development/LOCAL-DEVELOPMENT-HANDOFF.md) — ordered work queue for the local coding agent.
- [Cortile porting notes](development/CORTILE-PORTING.md) — behavior reference and license attribution.

## Testing and host validation
- [Testing strategy](testing/TESTING.md) — automated, Xvfb and manual test layers.
- [Host validation jobs](testing/HOST-VALIDATION-JOBS.md) — checks requiring Mint Cinnamon/Muffin.
- [Environment baseline](testing/environment-baseline.md) — reserved for recorded target-host evidence.

## User guides
- [Usage](user-guide/USAGE.md) — commands, configuration and limitations.
- [Installation/removal](user-guide/INSTALLATION.md) — development build and safe removal.

## Release engineering
- [Release checklist](release/RELEASING.md) — release acceptance and publishing.
- [Dependencies and attribution](release/DEPENDENCIES.md) — dependency inventory and notices.

Root-level contributor, security, conduct, changelog and agent files remain at the repository root as conventional entry points.
