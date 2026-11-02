# K-tile

K-tile is a Rust-based tiling-manager project designed to cooperate with existing Linux window managers. The primary target is Linux Mint 22.3 Cinnamon/Muffin on X11. A Wayland backend is a future goal, not a current capability.

K-tile takes functional inspiration from [Cortile](https://github.com/leukipp/cortile). Cortile is MIT-licensed. Any source code ported or adapted from it must preserve the applicable copyright and license notices. The current foundation is newly authored Rust code and is not feature-parity with Cortile.

## Status: early development

The current branch provides:
- a backend-independent domain model and deterministic layout planner;
- a pure reconciliation planner that proposes geometry changes and restoration only when the observed geometry still matches the last confirmed K-tile operation;
- vertical-left/right and horizontal-top/bottom master/stack layouts, maximized and grid plans;
- exclusion of floating, fullscreen, maximized, modal, above/below, skip-taskbar and skip-pager windows from layout plans;
- TOML parsing and validation;
- CLI commands for configuration initialization/validation and non-mutating layout previews;
- a read-only X11/EWMH adapter for client metadata and root-relative geometry, workspace/workarea information and RandR 1.5 monitor enumeration;
- a diagnostic X11 event watcher and a pure reconciliation planner with guarded restore proposals.

**K-tile does not yet automatically tile live desktop windows.** The event loop, safe geometry application/restoration, workspace and monitor management, rules, keyboard shortcuts, drag interactions, tray, full GTK4/Libadwaita GUI, and packaging remain unfinished. Do not configure the current development build to start automatically or treat it as a production tiling manager.

## Quick start

Requirements: stable Rust toolchain with Cargo, rustfmt and Clippy.

```sh
git clone https://github.com/D3M-Sudo/ktile.git
cd ktile
cargo build --release
cargo test --all-features
```

Example commands:

```sh
# Create a default config; existing files are not overwritten
target/release/k-tile init-config

# Validate the config
target/release/k-tile check-config

# Preview geometry without modifying desktop windows
target/release/k-tile plan --width 1920 --height 1080 --windows 5 --layout vertical-left
target/release/k-tile plan --width 1600 --height 900 --windows 6 --layout grid

# Inspect X11/EWMH connectivity (read-only)
target/release/k-tile check-x11
target/release/k-tile inspect-windows
RUST_LOG=info target/release/k-tile watch-x11
```

For all commands, configuration keys and limitations, see [Usage](docs/USAGE.md). For local build and removal instructions, see [Installation](docs/INSTALLATION.md).

## Development checks

Run the core quality gates with the Makefile:

```sh
make check
```

The equivalent individual commands are:

```sh
cargo fmt --all -- --check
cargo test --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release --all-features --locked
python3 scripts/check_dependency_licenses.py
```

The isolated X11 smoke test requires Xvfb, Openbox, x11-utils and xdotool:

```sh
make x11-integration
```

See [Testing](docs/TESTING.md), [Contributing](CONTRIBUTING.md) and [Agent instructions](AGENTS.md). CI results are available in [GitHub Actions](https://github.com/D3M-Sudo/ktile/actions).

## Documentation

- [Usage guide](docs/USAGE.md)
- [Installation and removal](docs/INSTALLATION.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Testing strategy and manual Cinnamon checklist](docs/TESTING.md)
- [Release checklist](docs/RELEASING.md)
- [Dependency and attribution inventory](docs/DEPENDENCIES.md)
- [Cortile porting and license notes](docs/CORTILE-PORTING.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Changelog](CHANGELOG.md)

## Safety contract

Future live geometry changes must revalidate the window's current state immediately before every request, confirm that the target remains eligible and inside the current monitor work area, and track the original geometry. Shutdown must not overwrite newer user-driven geometry. Tiling stays opt-in until integration tests and real Cinnamon/Muffin checks pass. Monitor dimensions must be discovered from the active session; no fixed resolution fallback is acceptable.

## License

K-tile is distributed under the MIT License; see [LICENSE](LICENSE). Third-party notices and the complete dependency-license audit are release requirements. See [LICENSES](LICENSES/README.md) and [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md).
