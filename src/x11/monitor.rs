// Lettura RandR heads + strut + workarea → WorkspaceArea (sola lettura).

use x11rb::protocol::randr::ConnectionExt as RandrConnectionExt;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

use crate::errors::KtileError;
use crate::layout::geometry::Rect;
use crate::x11::connection::EwmhAtoms;

/// Area geografica del monitor e area utile di lavoro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceArea {
    pub monitor_rect: Rect,
    pub usable_rect: Rect,
}

/// Legge la risoluzione del monitor tramite RandR e l'area utile tramite _NET_WORKAREA/strut.
/// Ritorna errore chiaro se vengono rilevati più monitor attivi (v1 singolo monitor).
pub fn read_workspace_area(
    conn: &RustConnection,
    root: Window,
    atoms: &EwmhAtoms,
) -> Result<WorkspaceArea, KtileError> {
    // 1. RandR heads
    let resources = conn
        .randr_get_screen_resources(root)
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    let mut active_crtc_count = 0;
    let mut primary_rect = None;

    for crtc in resources.crtcs {
        if let Ok(reply) = conn.randr_get_crtc_info(crtc, 0) {
            if let Ok(info) = reply.reply() {
                if info.width > 0 && info.height > 0 {
                    active_crtc_count += 1;
                    if primary_rect.is_none() {
                        primary_rect = Some(Rect {
                            x: info.x as i32,
                            y: info.y as i32,
                            width: info.width as u32,
                            height: info.height as u32,
                        });
                    }
                }
            }
        }
    }

    if active_crtc_count > 1 {
        return Err(KtileError::X11(
            "multi-monitor non supportato nella v1: rilevati più monitor attivi".to_string(),
        ));
    }

    let monitor_rect = primary_rect.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    });

    // 2. Area utile da _NET_WORKAREA
    let workarea_reply = conn
        .get_property(false, root, atoms.workarea, AtomEnum::CARDINAL, 0, 4)
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    let usable_rect = if let Some(mut iter) = workarea_reply.value32() {
        let x = iter.next().unwrap_or(0) as i32;
        let y = iter.next().unwrap_or(0) as i32;
        let w = iter.next().unwrap_or(monitor_rect.width);
        let h = iter.next().unwrap_or(monitor_rect.height);
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    } else {
        monitor_rect
    };

    Ok(WorkspaceArea {
        monitor_rect,
        usable_rect,
    })
}
