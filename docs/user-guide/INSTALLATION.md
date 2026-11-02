# Installation and removal

## Status
K-tile is in active development and is not ready for general installation as a live tiling manager. No system installer or release package is published. The current binary only previews layouts, validates configuration and inspects X11.

## Build locally
Prerequisites: stable Rust/Cargo, rustfmt and Clippy. An X11 session is required only for the X11 probe; Xvfb and a test window manager are useful for future integration tests.

    git clone https://github.com/D3M-Sudo/ktile.git
    cd ktile
    cargo build --release
    cargo test --all-features

Run target/release/k-tile directly. Do not enable autostart for a development build.

## Configuration and removal
Configuration is stored at $XDG_CONFIG_HOME/k-tile/config.toml or ~/.config/k-tile/config.toml. Init-config refuses to overwrite an existing file.

There is no system installer yet. Remove the repository checkout and, if created, the K-tile configuration directory after checking its exact path and backing up anything needed. Update this guide with exact install/uninstall and rollback instructions before packaging a release.
