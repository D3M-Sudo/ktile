# Usage guide

**Development status:** K-tile currently plans layouts, validates TOML configuration and performs read-only X11 checks. It does not automatically reposition live windows.

## Build
Install stable Rust with Cargo, rustfmt and Clippy. From a checkout run:

    cargo build --release

The executable is target/release/k-tile.

## Commands
Show help (also shown when no subcommand is supplied):

    target/release/k-tile --help

Create the default configuration. Existing files are not overwritten:

    target/release/k-tile init-config

Validate configuration:

    target/release/k-tile check-config
    target/release/k-tile --config ./config/config.example.toml check-config

Preview geometry without touching desktop windows:

    target/release/k-tile plan --width 1920 --height 1080 --windows 5 --layout vertical-left
    target/release/k-tile plan --width 1600 --height 900 --windows 6 --layout grid

Omit `--layout` to use the `layout` value from the TOML configuration:

    target/release/k-tile plan --width 1600 --height 900 --windows 6

Preview commands use validated settings from the TOML file when it exists; if no config file exists, the preview uses built-in defaults. Pass `--config PATH` to select another config. `check-config` still requires an existing config file.

Build a read-only preview using the real windows on the current X11 desktop. This command does not resize or move windows:

    target/release/k-tile plan-live

Override the layout for this preview:

    target/release/k-tile plan-live --layout grid

The preview uses the configured layout, master ratio, master count, gap and work-area policy. Floating and excluded application classes, plus fullscreen/maximized/modal/transient/dock/desktop windows, are omitted. Windows on other desktops are skipped; sticky windows remain eligible. If EWMH work-area metadata is unavailable, the command fails rather than guessing. Run `watch-plan` to refresh this preview automatically after relevant map/unmap/configure and EWMH root/client-property events; it still never changes window geometry. If RandR 1.5 monitor enumeration is unavailable, it logs a warning and uses the X11 server's actual root-screen geometry as a single-area fallback.

Check X11/EWMH connectivity:

    target/release/k-tile check-x11

Available layouts: vertical-left, vertical-right, horizontal-top, horizontal-bottom, maximized and grid. `check-x11` only inspects connectivity and screen dimensions; `inspect-windows` lists EWMH client metadata, and `watch-x11` is a diagnostic event watcher; `watch-plan` recomputes a read-only preview once per relevant event batch.

## Configuration
Default path is $XDG_CONFIG_HOME/k-tile/config.toml, or ~/.config/k-tile/config.toml if XDG_CONFIG_HOME is unset. See [the example configuration](../../config/config.example.toml).

Keys include layout, master_ratio (0.1–0.9), master_count (1–16), gap (0–128 pixels), respect_workarea, restore_on_exit, floating_classes, excluded_classes and log_level. Floating/excluded class entries match WM_CLASS case-insensitively. The configured log level is used unless overridden by RUST_LOG. The restore_on_exit policy is reserved for the future live-management milestone and has no effect while the backend remains read-only.

## Safe testing
Use preview commands first. Backend integration should use a disposable X11 session or Xvfb. Do not enable autostart or rely on the current build for live tiling. See [the testing strategy](../testing/TESTING.md).


### Cooperative shutdown

`watch-x11` and `watch-plan` handle SIGINT and SIGTERM. The signal handler only sets an atomic shutdown flag; the watcher checks it between event batches and during bounded 50 ms idle waits. Shutdown is clean and does not terminate Cinnamon/Muffin or alter client geometry. No automatic geometry restoration is performed because live geometry application is not yet enabled.
