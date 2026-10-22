use std::collections::HashSet;
use std::path::PathBuf;
use std::process::ExitCode;

use x11rb::connection::Connection;

use ktile::cli::CliArgs;
use ktile::config::Config;
use ktile::errors::KtileError;
use ktile::state::eligibility::{is_eligible, ExcludeConfig};
use ktile::state::reconciliation::{reconcile_read_only, take_snapshot};
use ktile::x11::applier::{apply_plan, ApplyStatus};
use ktile::x11::connection::{connect, EwmhAtoms};
use ktile::x11::ewmh;
use ktile::x11::monitor::read_workspace_area;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("ktile: {e}");
            ExitCode::from(e.exit_code())
        }
    }
}

fn resolve_config_path(explicit: Option<&PathBuf>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(p.clone());
    }
    let local_file = PathBuf::from("ktile.toml");
    if local_file.exists() {
        return Some(local_file);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let user_config = PathBuf::from(home).join(".config/ktile/ktile.toml");
        if user_config.exists() {
            return Some(user_config);
        }
    }
    None
}

fn load_config(explicit_path: Option<&PathBuf>) -> Result<Config, KtileError> {
    let resolved = resolve_config_path(explicit_path);
    match resolved {
        Some(path) => Config::load_from_file(&path).map_err(|e| {
            eprintln!("ktile: {e}");
            KtileError::ConfigInvalid(e.to_string())
        }),
        None => {
            if let Some(p) = explicit_path {
                return Err(KtileError::ConfigInvalid(format!(
                    "File di configurazione non trovato: {}",
                    p.display()
                )));
            }
            let mut cfg = Config::default();
            cfg.validate()
                .map_err(|e| KtileError::ConfigInvalid(e.to_string()))?;
            Ok(cfg)
        }
    }
}

fn run() -> Result<u8, KtileError> {
    let cli = CliArgs::parse_args();

    // 1. --check-config: No X11 connection needed
    if cli.check_config {
        let mut cfg = load_config(cli.config.as_ref())?;
        if let Some(ref layout_override) = cli.layout {
            match layout_override.as_str() {
                "dwindle" => cfg.general.layout = ktile::config::LayoutType::Dwindle,
                "master-stack" => cfg.general.layout = ktile::config::LayoutType::MasterStack,
                _ => {}
            }
        }
        println!("Configurazione valida:");
        println!("{:#?}", cfg);
        return Ok(0);
    }

    // 2. Check X11 session
    if !cli.has_display() {
        return Err(KtileError::NoDisplay);
    }

    // Load config for execution
    let mut config = load_config(cli.config.as_ref())?;
    if let Some(ref layout_override) = cli.layout {
        match layout_override.as_str() {
            "dwindle" => config.general.layout = ktile::config::LayoutType::Dwindle,
            "master-stack" => config.general.layout = ktile::config::LayoutType::MasterStack,
            _ => {}
        }
    }

    // 3. X11 connection & EWMH sanity checks
    let (conn, screen_num) = connect()?;
    let atoms = EwmhAtoms::new(&conn)?;
    let root = conn.setup().roots[screen_num].root;

    let wm_name = ewmh::verify_wm(&conn, root, &atoms)?;

    // 4. --list-windows
    if cli.list_windows {
        let current_desktop = ewmh::get_current_desktop(&conn, root, &atoms)?;
        let snapshot = take_snapshot(&conn, root, &atoms, current_desktop)?;
        println!("WM Rilevato: {}", wm_name);
        println!("Desktop Corrente: {}", current_desktop);
        println!("Finestre rilevate (totale {}):", snapshot.windows.len());

        let exclude_cfg = ExcludeConfig {
            classes: config.exclude.classes.clone(),
            instances: config.exclude.instances.clone(),
        };

        for item in &snapshot.windows {
            let eligible =
                !item.is_hidden && is_eligible(&item.meta, current_desktop, &exclude_cfg);
            println!(
                "  [0x{:x}] class='{}' instance='{}' desktop={} hidden={} eligible={}",
                item.window,
                item.meta.class.as_deref().unwrap_or("N/A"),
                item.meta.instance.as_deref().unwrap_or("N/A"),
                item.meta.desktop,
                item.is_hidden,
                eligible
            );
        }
        return Ok(0);
    }

    // 5. Layout planning (dry-run or default execution)
    let current_desktop = ewmh::get_current_desktop(&conn, root, &atoms)?;
    let area = read_workspace_area(&conn, root, &atoms)?;
    let snapshot = take_snapshot(&conn, root, &atoms, current_desktop)?;
    let plan = reconcile_read_only(&snapshot, &config, &area)?;

    println!("Ktile Read-Only Planning:");
    println!("  WM: {}", wm_name);
    println!("  Layout: {:?}", config.general.layout);
    println!("  Usable Area: {:?}", area.usable_rect);
    println!("  Mosse calcolate (totale {}):", plan.moves.len());
    for m in &plan.moves {
        println!("    Window 0x{:x} -> {:?}", m.window_id.0, m.target_rect);
    }

    if cli.dry_run || !cli.allow_apply {
        println!("OK: 0 X11 write requests sent (read-only / dry-run mode).");
        if !cli.dry_run && !cli.allow_apply {
            println!("Per applicare le geometrie, usare esplicitamente --allow-apply (M5).");
        }
    } else {
        // The plan was built from this fresh snapshot. Only IDs in the plan
        // can pass the applier's allow-list; the applier rechecks live state
        // immediately before each request.
        let allowed: HashSet<_> = plan
            .moves
            .iter()
            .map(|movement| movement.window_id)
            .collect();
        let report = apply_plan(&conn, &plan, &allowed, current_desktop, &atoms)?;
        println!("M5 geometry application report:");
        println!("  X11 write requests sent: {}", report.requests_sent);
        println!("  Applied/already correct: {}", report.applied_count());
        println!("  WM geometry deviations: {}", report.deviated_count());
        println!("  Skipped by safety gate: {}", report.skipped_count());
        println!("  Failed requests/readback: {}", report.failed_count());
        for item in report.outcomes {
            match item.status {
                ApplyStatus::Applied { actual } => println!("  [APPLIED] 0x{:x}: {:?}", item.window_id.0, actual),
                ApplyStatus::AlreadyCorrect => println!("  [OK] 0x{:x}: already at target", item.window_id.0),
                ApplyStatus::Deviated { actual } => eprintln!("  [DEVIATION] 0x{:x}: requested {:?}, actual {:?}", item.window_id.0, item.target, actual),
                ApplyStatus::Skipped { reason } => eprintln!("  [SKIPPED] 0x{:x}: {}", item.window_id.0, reason),
                ApplyStatus::Failed { reason } => eprintln!("  [FAILED] 0x{:x}: {}", item.window_id.0, reason),
            }
        }
    }

    Ok(0)
}
