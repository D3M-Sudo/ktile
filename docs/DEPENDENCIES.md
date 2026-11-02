# Dependency and attribution inventory

This initial inventory is not a complete legal report. Re-check the full resolved dependency graph for each release.

## Direct Rust dependencies

| Crate | Purpose |
|---|---|
| clap | CLI argument parsing |
| serde | Serialization and configuration model |
| toml | TOML parsing and serialization |
| thiserror | Typed errors |
| tracing | Structured logging API |
| tracing-subscriber | Logging subscriber and environment filters |
| x11rb (optional x11 feature) | X11 protocol access |
| proptest (development) | Property-based tests |
| tempfile (development) | Temporary test files |

Cargo version constraints are not exact resolved versions. Review Cargo.lock and generate a full direct/transitive license report before distribution.

## Upstream source reuse
Cortile: https://github.com/leukipp/cortile

Cortile is MIT-licensed; its upstream license includes copyright notices for Berin Larson (2019) and leukipp (2022). The current K-tile foundation is newly authored and does not claim a completed source port. Before importing code, record source paths, modifications, original notices and applicable license text in CORTILE-PORTING.md.

## Release requirements
- [ ] Commit and review Cargo.lock.
- [ ] Generate a complete dependency license inventory.
- [ ] Check advisories and unsupported dependencies.
- [ ] Include required license texts/notices in source and binary distributions.
- [ ] Re-run checks when dependencies change.
