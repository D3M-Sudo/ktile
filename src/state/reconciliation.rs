use x11rb::protocol::xproto::Window;
use x11rb::rust_connection::RustConnection;

use crate::config::Config;
use crate::errors::KtileError;
use crate::layout::geometry::{Gaps, SplitMode};
use crate::layout::plan::{Plan, WindowMove};
use crate::state::eligibility::{
    is_eligible, ExcludeConfig, WindowMeta, WindowStateAtom, WindowType,
};
use crate::state::registry::WindowId;
use crate::x11::connection::EwmhAtoms;
use crate::x11::ewmh;
use crate::x11::icccm::{self, WindowClass};
use crate::x11::monitor::WorkspaceArea;

pub struct SnapshotWindow {
    pub window: Window,
    pub meta: WindowMeta,
    pub class: Option<WindowClass>,
    pub is_hidden: bool,
}

pub struct SystemSnapshot {
    pub active_desktop: u32,
    pub active_window: Option<Window>,
    pub windows: Vec<SnapshotWindow>,
}

/// Reads the current system state from X11 in read-only mode.
pub fn take_snapshot(
    conn: &RustConnection,
    root: Window,
    atoms: &EwmhAtoms,
    current_desktop: u32,
) -> Result<SystemSnapshot, KtileError> {
    let active_window = ewmh::get_active_window(conn, root, atoms)?;
    let client_list = ewmh::client_list(conn, root, atoms)?;

    let mut windows = Vec::new();

    for w in client_list {
        let ewmh_type = ewmh::get_wm_window_type(conn, atoms, w).ok().flatten();
        let ewmh_states = ewmh::get_wm_state(conn, atoms, w).unwrap_or_default();
        let desktop = ewmh::get_wm_desktop(conn, atoms, w)
            .ok()
            .flatten()
            .unwrap_or(0);
        let transient_for = icccm::read_wm_transient_for(conn, w)
            .ok()
            .flatten()
            .map(WindowId);
        let class_info = icccm::read_wm_class(conn, w).ok().flatten();

        let class = class_info.as_ref().map(|c| c.class_name.clone());
        let instance = class_info.as_ref().map(|c| c.instance.clone());

        // Map EWMH window type to internal WindowType
        let window_type = ewmh_type.map(|t| {
            if t == atoms.wm_window_type_normal {
                WindowType::Normal
            } else if t == atoms.wm_window_type_dialog {
                WindowType::Dialog
            } else if t == atoms.wm_window_type_dock {
                WindowType::Dock
            } else {
                WindowType::Unknown
            }
        });

        // Map EWMH states
        let mut states = Vec::new();
        let is_hidden = ewmh_states.contains(&atoms.wm_state_hidden);

        if ewmh_states.contains(&atoms.wm_state_fullscreen) {
            states.push(WindowStateAtom::Fullscreen);
        }
        if is_hidden {
            states.push(WindowStateAtom::Hidden);
        }

        let meta = WindowMeta {
            id: WindowId(w),
            is_mapped: true,
            window_type,
            transient_for,
            states,
            desktop,
            class,
            instance,
            title: None,
        };

        windows.push(SnapshotWindow {
            window: w,
            meta,
            class: class_info,
            is_hidden,
        });
    }

    Ok(SystemSnapshot {
        active_desktop: current_desktop,
        active_window,
        windows,
    })
}

/// Computes the layout plan in read-only mode by comparing snapshot vs current state.
pub fn reconcile_read_only(
    snapshot: &SystemSnapshot,
    config: &Config,
    area: &WorkspaceArea,
) -> Result<Plan, KtileError> {
    let exclude_config = ExcludeConfig {
        classes: config.exclude.classes.clone(),
        instances: config.exclude.instances.clone(),
    };

    let mut eligible_windows = Vec::new();

    for item in &snapshot.windows {
        if !item.is_hidden && is_eligible(&item.meta, snapshot.active_desktop, &exclude_config) {
            eligible_windows.push(item.window);
        }
    }

    if eligible_windows.is_empty() {
        return Ok(Plan::default());
    }

    let gaps = Gaps {
        inner: config.gaps.inner,
        outer: config.gaps.outer,
    };

    let rects = match config.general.layout {
        crate::config::LayoutType::Dwindle => {
            let mode = match config.dwindle.split_mode {
                crate::config::SplitMode::LongestSide => SplitMode::LongestSide,
                crate::config::SplitMode::Alternating => SplitMode::Alternating,
            };
            crate::layout::geometry::compute_dwindle(
                area.usable_rect,
                &gaps,
                eligible_windows.len(),
                mode,
                config.dwindle.split_ratio as f64,
            )
        }
        crate::config::LayoutType::MasterStack => crate::layout::geometry::compute_master_stack(
            area.usable_rect,
            &gaps,
            eligible_windows.len(),
            config.master.mfact as f64,
        ),
    };

    let mut moves = Vec::new();
    for (w, rect) in eligible_windows.into_iter().zip(rects) {
        moves.push(WindowMove {
            window_id: WindowId(w),
            target_rect: rect,
        });
    }

    Ok(Plan::new(moves))
}
