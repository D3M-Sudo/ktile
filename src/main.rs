use clap::{Parser, Subcommand};
use k_tile::{
    config::Config,
    layout::{LayoutEngine, LayoutKind, Rect},
};
use std::{path::PathBuf, process::ExitCode};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "k-tile",
    version,
    about = "Modern lightweight X11 tiling manager for Cinnamon/Muffin"
)]
struct Cli {
    /// Path to TOML configuration.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a TOML configuration without connecting to X11.
    CheckConfig,
    /// Write a default TOML configuration; refuses to overwrite existing files.
    InitConfig,
    /// Print a deterministic sample layout plan without touching desktop windows.
    Plan {
        #[arg(long, default_value_t = 1920)]
        width: u32,
        #[arg(long, default_value_t = 1080)]
        height: u32,
        #[arg(long, default_value_t = 4)]
        windows: u32,
        #[arg(long, value_enum, default_value_t = LayoutArg::VerticalLeft)]
        layout: LayoutArg,
    },
    /// Inspect whether an EWMH-compatible X11 window manager is reachable.
    CheckX11,
    /// List EWMH client windows and read their metadata without changing desktop state.
    InspectWindows,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum LayoutArg {
    VerticalLeft,
    VerticalRight,
    HorizontalTop,
    HorizontalBottom,
    Maximized,
    Grid,
}

impl From<LayoutArg> for LayoutKind {
    fn from(value: LayoutArg) -> Self {
        match value {
            LayoutArg::VerticalLeft => Self::VerticalLeft,
            LayoutArg::VerticalRight => Self::VerticalRight,
            LayoutArg::HorizontalTop => Self::HorizontalTop,
            LayoutArg::HorizontalBottom => Self::HorizontalBottom,
            LayoutArg::Maximized => Self::Maximized,
            LayoutArg::Grid => Self::Grid,
        }
    }
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(default_config_path);

    match run(cli.command, &config_path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("k-tile: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(
    command: Option<Command>,
    config_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    match command.unwrap_or(Command::CheckConfig) {
        Command::CheckConfig => {
            Config::load(config_path)?;
            println!("Configuration is valid: {}", config_path.display());
        }
        Command::InitConfig => {
            if config_path.exists() {
                return Err(format!("refusing to overwrite {}", config_path.display()).into());
            }
            Config::write_default(config_path)?;
            println!("Wrote default configuration: {}", config_path.display());
        }
        Command::Plan {
            width,
            height,
            windows,
            layout,
        } => {
            let infos: Vec<_> = (1..=windows)
                .map(|id| k_tile::WindowInfo {
                    id: k_tile::WindowId(id),
                    title: Some(format!("Example {id}")),
                    app_class: None,
                    workspace: 0,
                    monitor: 0,
                    state: Default::default(),
                    floating: false,
                })
                .collect();
            let plan = LayoutEngine::default().plan(
                layout.into(),
                Rect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
                &infos,
            )?;
            println!(
                "layout={:?} area={}x{} windows={}",
                plan.layout,
                width,
                height,
                plan.windows.len()
            );
            for window in plan.windows {
                println!(
                    "window={} x={} y={} width={} height={}",
                    window.id.0,
                    window.rect.x,
                    window.rect.y,
                    window.rect.width,
                    window.rect.height
                );
            }
        }
        Command::CheckX11 => {
            #[cfg(feature = "x11")]
            {
                let backend = k_tile::x11::X11Backend::connect()?;
                let (width, height) = backend.screen_size();
                println!(
                    "X11 connection OK; root={} screen={}x{}",
                    backend.root_window(),
                    width,
                    height
                );
                let current = backend.current_desktop()?;
                println!(
                    "current_desktop={:?} desktop_count={:?}",
                    current,
                    backend.desktop_count()?
                );
                if let Some(desktop) = current {
                    if let Some(area) = backend.work_area(desktop)? {
                        println!(
                            "workarea x={} y={} width={} height={}",
                            area.x, area.y, area.width, area.height
                        );
                    }
                }
            }
            #[cfg(not(feature = "x11"))]
            return Err("this build has no X11 support".into());
        }
        Command::InspectWindows => {
            #[cfg(feature = "x11")]
            {
                let backend = k_tile::x11::X11Backend::connect()?;
                let windows = backend.client_windows()?;
                if windows.is_empty() {
                    println!("No EWMH client windows found.");
                }
                for id in windows {
                    let title = backend.window_title(id).unwrap_or_else(|error| {
                        tracing::warn!(window = id.0, %error, "could not read window title");
                        None
                    });
                    let class = backend.window_class(id).unwrap_or_else(|error| {
                        tracing::warn!(window = id.0, %error, "could not read window class");
                        None
                    });
                    let desktop = backend.window_desktop(id).unwrap_or_else(|error| {
                        tracing::warn!(window = id.0, %error, "could not read window desktop");
                        None
                    });
                    let state = match backend.window_state(id) {
                        Ok(state) => state,
                        Err(error) => {
                            tracing::warn!(window = id.0, %error, "could not read window state");
                            continue;
                        }
                    };
                    // Debug formatting escapes control characters in untrusted titles/classes.
                    println!(
                        "window={} desktop={:?} class={:?} title={:?} state={:?}",
                        id.0, desktop, class, title, state
                    );
                }
            }
            #[cfg(not(feature = "x11"))]
            return Err("this build has no X11 support".into());
        }
    }
    Ok(())
}

fn default_config_path() -> PathBuf {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home).join("k-tile/config.toml");
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/k-tile/config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_path_is_namespaced() {
        assert!(default_config_path().ends_with("k-tile/config.toml"));
    }
}
