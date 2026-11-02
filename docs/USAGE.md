# Usage guide

**Development status:** K-tile currently plans layouts, validates TOML configuration and performs read-only X11 checks. It does not automatically reposition live windows.

## Build
Install stable Rust with Cargo, rustfmt and Clippy. From a checkout run:

    cargo build --release

The executable is target/release/k-tile.

## Commands
Show help:

    target/release/k-tile --help

Create the default configuration. Existing files are not overwritten:

    target/release/k-tile init-config

Validate configuration:

    target/release/k-tile check-config
    target/release/k-tile --config ./config.example.toml check-config

Preview geometry without touching desktop windows:

    target/release/k-tile plan --width 1920 --height 1080 --windows 5 --layout vertical-left
    target/release/k-tile plan --width 1600 --height 900 --windows 6 --layout grid
# Omit --layout to use the `layout` value from the TOML configuration:
target/release/k-tile plan --width 1600 --height 900 --windows 6

Preview commands use validated settings from the TOML file when it exists; if no config file exists, the preview uses built-in defaults. Pass `--config PATH` to select another config. `check-config` still requires an existing config file.

Build a read-only preview using the real windows on the current X11 desktop. This command does not resize or move windows:

    target/release/k-tile plan-live

Override the layout for this preview:

    target/release/k-tile plan-live --layout grid

The preview uses the configured layout, master ratio, master count, gap and work-area policy. Floating and excluded application classes are omitted. Windows on other desktops are skipped; sticky windows remain eligible. It fails rather than guessing the work area when the window manager does not publish the required EWMH metadata.

Check X11/EWMH connectivity:

    target/release/k-tile check-x11

Available layouts: vertical-left, vertical-right, horizontal-top, horizontal-bottom, maximized and grid. The X11 command only inspects connectivity and screen dimensions.

## Configuration
Default path is $XDG_CONFIG_HOME/k-tile/config.toml, or ~/.config/k-tile/config.toml if XDG_CONFIG_HOME is unset. See config.example.toml.

Keys include layout, master_ratio (0.1–0.9), master_count (1–16), gap (0–128 pixels), respect_workarea, restore_on_exit, floating_classes, excluded_classes and log_level. Some policy fields are stored for future milestones and do not yet affect live windows.

## Safe testing
Use preview commands first. Backend integration should use a disposable X11 session or Xvfb. Do not enable autostart or rely on the current build for live tiling. See TESTING.md.
