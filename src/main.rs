use clap::{Parser, Subcommand};
use k_tile::{
    config::Config,
    layout::{LayoutKind, Rect},
};
use std::{path::PathBuf, process::ExitCode};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "k-tile",
    version,
    about = "Modern lightweight X11 tiling manager for Cinnamon/Muffin",
    subcommand_required = true,
    arg_required_else_help = true
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
        #[arg(long, value_enum)]
        layout: Option<LayoutArg>,
    },
    /// Build a read-only layout preview from currently managed X11 windows.
    PlanLive {
        #[arg(long, value_enum)]
        layout: Option<LayoutArg>,
    },
    /// Inspect whether an EWMH-compatible X11 window manager is reachable.
    CheckX11,
    /// List EWMH client windows and read their metadata without changing desktop state.
    InspectWindows,
    /// Watch basic X11 window lifecycle events without changing window geometry.
    WatchX11,
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
    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(default_config_path);
    let configured_level = if config_path.exists() {
        Config::load(&config_path)
            .map(|config| config.log_level)
            .unwrap_or_else(|_| "info".into())
    } else {
        "info".into()
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(configured_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();

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
            let config = if config_path.exists() {
                Config::load(config_path)?
            } else {
                Config::default()
            };
            let layout = layout.map(Into::into).unwrap_or(config.layout);
            let plan = config.engine().plan(
                layout,
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
        Command::PlanLive { layout } => {
            #[cfg(feature = "x11")]
            {
                let config = if config_path.exists() {
                    Config::load(config_path)?
                } else {
                    Config::default()
                };
                let backend = k_tile::x11::X11Backend::connect()?;
                let desktop = backend
                    .current_desktop()?
                    .ok_or("window manager does not publish _NET_CURRENT_DESKTOP")?;
                let monitor_specs: Vec<(u32, String, Rect)> = match backend.monitors() {
                    Ok(monitors) if !monitors.is_empty() => monitors
                        .into_iter()
                        .enumerate()
                        .map(|(index, monitor)| {
                            (
                                index as u32,
                                monitor.name,
                                Rect {
                                    x: monitor.x,
                                    y: monitor.y,
                                    width: monitor.width,
                                    height: monitor.height,
                                },
                            )
                        })
                        .collect(),
                    Ok(_) => {
                        tracing::warn!(
                            "RandR returned no monitors; using X11 root geometry for this preview"
                        );
                        let (width, height) = backend.screen_size();
                        vec![(
                            0,
                            "X11 root screen".into(),
                            Rect {
                                x: 0,
                                y: 0,
                                width: u32::from(width),
                                height: u32::from(height),
                            },
                        )]
                    }
                    Err(error) => {
                        tracing::warn!(%error, "RandR monitor discovery unavailable; using X11 root geometry for this preview");
                        let (width, height) = backend.screen_size();
                        vec![(
                            0,
                            "X11 root screen".into(),
                            Rect {
                                x: 0,
                                y: 0,
                                width: u32::from(width),
                                height: u32::from(height),
                            },
                        )]
                    }
                };
                let work_area = if config.respect_workarea {
                    Some(
                        backend
                            .work_area(desktop)?
                            .ok_or("window manager does not publish a valid _NET_WORKAREA")?,
                    )
                } else {
                    None
                };
                let monitor_areas: Vec<(u32, String, Rect)> = monitor_specs
                    .iter()
                    .filter_map(|(id, name, monitor_rect)| {
                        let area = work_area
                            .map(|work| intersect_rect(*monitor_rect, work))
                            .unwrap_or(Some(*monitor_rect));
                        area.map(|area| (*id, name.clone(), area))
                    })
                    .collect();
                if monitor_areas.is_empty() {
                    return Err("no usable monitor work areas are available".into());
                }
                let mut windows = Vec::new();
                for id in backend.client_windows()? {
                    let state = match backend.window_state(id) {
                        Ok(state) => state,
                        Err(error) => {
                            tracing::warn!(window = id.0, %error, "skipping window with unreadable state");
                            continue;
                        }
                    };
                    let window_desktop = match backend.window_desktop(id) {
                        Ok(value) => value,
                        Err(error) => {
                            tracing::warn!(window = id.0, %error, "skipping window with unreadable desktop");
                            continue;
                        }
                    };
                    if !belongs_to_desktop(window_desktop, desktop, state.sticky) {
                        continue;
                    }
                    let app_class = match backend.window_class(id) {
                        Ok(value) => value,
                        Err(error) => {
                            tracing::warn!(window = id.0, %error, "skipping window with unreadable class");
                            continue;
                        }
                    };
                    let title = backend.window_title(id).unwrap_or_else(|error| {
                        tracing::warn!(window = id.0, %error, "window title unavailable");
                        None
                    });
                    let class_matches = |patterns: &[String]| {
                        app_class.as_ref().is_some_and(|class| {
                            patterns
                                .iter()
                                .any(|pattern| class.eq_ignore_ascii_case(pattern))
                        })
                    };
                    let floating = class_matches(&config.floating_classes)
                        || class_matches(&config.excluded_classes);
                    let geometry = match backend.window_geometry(id) {
                        Ok(geometry) => geometry,
                        Err(error) => {
                            tracing::warn!(window = id.0, %error, "skipping window with unreadable geometry");
                            continue;
                        }
                    };
                    let center_x = i64::from(geometry.x) + i64::from(geometry.width) / 2;
                    let center_y = i64::from(geometry.y) + i64::from(geometry.height) / 2;
                    let Some((monitor_id, _, _)) = monitor_specs.iter().find(|(_, _, rect)| {
                        center_x >= i64::from(rect.x)
                            && center_x < rect.right()
                            && center_y >= i64::from(rect.y)
                            && center_y < rect.bottom()
                    }) else {
                        tracing::warn!(
                            window = id.0,
                            "skipping window whose center is outside known monitors"
                        );
                        continue;
                    };
                    windows.push(k_tile::WindowInfo {
                        id,
                        title,
                        app_class,
                        workspace: window_desktop.unwrap_or(desktop),
                        monitor: *monitor_id,
                        state,
                        floating,
                    });
                }
                let selected_layout = layout.map(Into::into).unwrap_or(config.layout);
                let engine = config.engine();
                let mut total_planned = 0usize;
                println!(
                    "read-only live preview: desktop={} layout={:?} monitors={}",
                    desktop,
                    selected_layout,
                    monitor_areas.len()
                );
                for (monitor_id, monitor_name, area) in &monitor_areas {
                    let monitor_windows: Vec<_> = windows
                        .iter()
                        .filter(|window| window.monitor == *monitor_id)
                        .cloned()
                        .collect();
                    let plan = engine.plan(selected_layout, *area, &monitor_windows)?;
                    total_planned += plan.windows.len();
                    println!(
                        "monitor={} name={:?} area={}x{} at {},{} eligible_windows={}",
                        monitor_id,
                        monitor_name,
                        area.width,
                        area.height,
                        area.x,
                        area.y,
                        plan.windows.len()
                    );
                    for planned in plan.windows {
                        let window = monitor_windows
                            .iter()
                            .find(|window| window.id == planned.id);
                        println!(
                            "window={} monitor={} class={:?} title={:?} x={} y={} width={} height={}",
                            planned.id.0,
                            monitor_id,
                            window.and_then(|window| window.app_class.as_deref()),
                            window.and_then(|window| window.title.as_deref()),
                            planned.rect.x,
                            planned.rect.y,
                            planned.rect.width,
                            planned.rect.height
                        );
                    }
                }
                println!("Planned windows: {total_planned}. No window geometry was changed.");
            }
            #[cfg(not(feature = "x11"))]
            return Err("this build has no X11 support".into());
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
                match backend.monitors() {
                    Ok(monitors) => {
                        for monitor in monitors {
                            println!(
                                "monitor={:?} primary={} x={} y={} width={} height={}",
                                monitor.name,
                                monitor.primary,
                                monitor.x,
                                monitor.y,
                                monitor.width,
                                monitor.height
                            );
                        }
                    }
                    Err(error) => {
                        eprintln!("RandR monitor enumeration unavailable: {error}");
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
                    let geometry = match backend.window_geometry(id) {
                        Ok(geometry) => geometry,
                        Err(error) => {
                            tracing::warn!(window = id.0, %error, "could not read window geometry");
                            continue;
                        }
                    };
                    // Debug formatting escapes control characters in untrusted titles/classes.
                    println!(
                        "window={} desktop={:?} geometry={:?} class={:?} title={:?} state={:?}",
                        id.0, desktop, geometry, class, title, state
                    );
                }
            }
            #[cfg(not(feature = "x11"))]
            return Err("this build has no X11 support".into());
        }
        Command::WatchX11 => {
            #[cfg(feature = "x11")]
            {
                k_tile::x11::X11Backend::connect()?.watch_events()?;
            }
            #[cfg(not(feature = "x11"))]
            return Err("this build has no X11 support".into());
        }
    }
    Ok(())
}

fn belongs_to_desktop(window_desktop: Option<u32>, current_desktop: u32, sticky: bool) -> bool {
    sticky
        || window_desktop.is_none()
        || window_desktop.is_some_and(|desktop| desktop == current_desktop || desktop == u32::MAX)
}

fn intersect_rect(a: Rect, b: Rect) -> Option<Rect> {
    let left = i64::from(a.x).max(i64::from(b.x));
    let top = i64::from(a.y).max(i64::from(b.y));
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());
    if right <= left || bottom <= top {
        return None;
    }
    Some(Rect {
        x: i32::try_from(left).ok()?,
        y: i32::try_from(top).ok()?,
        width: u32::try_from(right - left).ok()?,
        height: u32::try_from(bottom - top).ok()?,
    })
}

fn default_config_path() -> PathBuf {
    config_path_from_env(
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn config_path_from_env(xdg_config_home: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    if let Some(config_home) = xdg_config_home.filter(|path| path.is_absolute()) {
        return config_home.join("k-tile/config.toml");
    }
    home.unwrap_or_else(|| PathBuf::from("."))
        .join(".config/k-tile/config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sticky_windows_are_not_filtered_by_desktop_number() {
        assert!(belongs_to_desktop(Some(3), 1, true));
    }

    #[test]
    fn ordinary_windows_on_other_desktops_are_filtered() {
        assert!(!belongs_to_desktop(Some(3), 1, false));
        assert!(belongs_to_desktop(Some(1), 1, false));
        assert!(belongs_to_desktop(Some(u32::MAX), 1, false));
        assert!(belongs_to_desktop(None, 1, false));
    }

    #[test]
    fn intersects_monitor_with_global_work_area() {
        let monitor = Rect {
            x: -1600,
            y: 0,
            width: 1600,
            height: 900,
        };
        let work = Rect {
            x: -1600,
            y: 24,
            width: 3200,
            height: 876,
        };
        assert_eq!(
            intersect_rect(monitor, work),
            Some(Rect {
                x: -1600,
                y: 24,
                width: 1600,
                height: 876,
            })
        );
    }

    #[test]
    fn returns_none_for_disjoint_monitor_and_work_area() {
        let monitor = Rect {
            x: -1600,
            y: 0,
            width: 1600,
            height: 900,
        };
        let work = Rect {
            x: 0,
            y: 24,
            width: 1920,
            height: 876,
        };
        assert_eq!(intersect_rect(monitor, work), None);
    }

    #[test]
    fn default_config_path_is_namespaced() {
        assert!(default_config_path().ends_with("k-tile/config.toml"));
    }

    #[test]
    fn config_path_uses_absolute_xdg_config_home() {
        assert_eq!(
            config_path_from_env(
                Some(PathBuf::from("/tmp/user-config")),
                Some(PathBuf::from("/home/test"))
            ),
            PathBuf::from("/tmp/user-config/k-tile/config.toml")
        );
    }

    #[test]
    fn config_path_ignores_relative_xdg_config_home() {
        assert_eq!(
            config_path_from_env(
                Some(PathBuf::from("relative-config")),
                Some(PathBuf::from("/home/test"))
            ),
            PathBuf::from("/home/test/.config/k-tile/config.toml")
        );
    }

    #[test]
    fn config_path_falls_back_to_dot_config_when_home_is_unset() {
        assert_eq!(
            config_path_from_env(None, None),
            PathBuf::from("./.config/k-tile/config.toml")
        );
    }

    #[test]
    fn cli_requires_a_subcommand_instead_of_silently_checking_config() {
        assert!(Cli::try_parse_from(["k-tile"]).is_err());
    }

    #[test]
    fn plan_live_command_is_available_and_layout_override_is_optional() {
        let cli = Cli::try_parse_from(["k-tile", "plan-live"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::PlanLive { layout: None })
        ));
        let explicit = Cli::try_parse_from(["k-tile", "plan-live", "--layout", "grid"]).unwrap();
        assert!(matches!(
            explicit.command,
            Some(Command::PlanLive {
                layout: Some(LayoutArg::Grid)
            })
        ));
    }

    #[test]
    fn layout_arg_is_optional_so_toml_layout_can_be_used() {
        let cli = Cli::try_parse_from(["k-tile", "plan"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Plan { layout: None, .. })
        ));
        let explicit = Cli::try_parse_from(["k-tile", "plan", "--layout", "grid"]).unwrap();
        assert!(matches!(
            explicit.command,
            Some(Command::Plan {
                layout: Some(LayoutArg::Grid),
                ..
            })
        ));
    }
}
