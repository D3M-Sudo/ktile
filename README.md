# K-tile

K-tile is a Rust-based dynamic tiling manager for Linux desktop environments that expose the X11 EWMH/ICCCM interfaces. The first supported target is Linux Mint 22.3 Cinnamon running X11 with Muffin. A future Wayland backend is an architectural goal, not a current capability.

The project takes functional inspiration from [Cortile](https://github.com/leukipp/cortile). Cortile is MIT-licensed; any source code ported from it must retain its applicable copyright and license notices. K-tile's initial implementation is an independent Rust foundation, not a claim of feature parity.

## Current implementation status

This repository is under active reconstruction. The initial foundation provides:
- a backend-independent geometry model and deterministic layout planner;
- vertical-left/right and horizontal-top/bottom master/stack layouts, maximized and grid layouts;
- conservative exclusion of floating, fullscreen, maximized, modal, above/below and skip-taskbar/pager windows from plans;
- TOML parsing and validation;
- CLI commands for config initialization/validation, sample layout planning and read-only X11 connection checks;
- a read-only X11 connection probe.

**It does not yet automatically tile live desktop windows.** The event loop, EWMH state reconciliation, geometry application with revalidation, workspace/monitor tracking, GUI, tray integration, drag-and-drop, hotkeys and safe restore workflow remain implementation milestones.

## Build

Requirements: Rust toolchain and (for X11 integration) X11 development/runtime libraries as required by the target environment.

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

## Usage

```sh
# Validate a configuration
cargo run -- check-config --config ~/.config/k-tile/config.toml

# Create a default config (does not overwrite an existing file)
cargo run -- --config ~/.config/k-tile/config.toml init-config

# Preview geometry without changing any desktop windows
cargo run -- plan --width 1920 --height 1080 --windows 5 --layout grid

# Read-only X11/EWMH connection check
cargo run -- check-x11
```

The CLI is deliberately read-only at this stage. No window geometry is changed by the current implementation.

## Configuration

See [config.example.toml](config.example.toml). The default config location is `$XDG_CONFIG_HOME/k-tile/config.toml`, or `~/.config/k-tile/config.toml` if `XDG_CONFIG_HOME` is not set.

## Architecture and roadmap

- [Architecture](docs/ARCHITECTURE.md)
- [Development roadmap](docs/ROADMAP.md)
- [Cortile porting and license notes](docs/CORTILE-PORTING.md)

## Safety contract

The live backend must never resize a window based only on a stale snapshot. Before every geometry request, it must re-read the target's relevant EWMH state, verify the window is still eligible and confirm that the request remains within the current monitor work area. Tiling remains opt-in until target-machine tests on Cinnamon/Muffin pass. Disabling K-tile must restore captured geometry where the window still exists and where restoration will not overwrite a newer user action.

## License

K-tile is distributed under the MIT License. See [LICENSE](LICENSE). Source code copied or adapted from Cortile must preserve Cortile's copyright and license notices.
