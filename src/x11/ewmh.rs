// Lettura proprietà EWMH (read-only): doppia verifica WM check, liste client, active window.

use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use crate::errors::KtileError;
use crate::x11::connection::EwmhAtoms;

/// Verifica EWMH essenziale: lettura `_NET_SUPPORTING_WM_CHECK` su root → XID,
/// rilettura sulla finestra WM + `_NET_WM_NAME`. Assente → exit 6 (D6).
pub fn verify_wm(
    conn: &RustConnection,
    root: Window,
    atoms: &EwmhAtoms,
) -> Result<String, KtileError> {
    let reply = conn
        .get_property(
            false,
            root,
            atoms.supporting_wm_check,
            AtomEnum::WINDOW,
            0,
            1,
        )
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    let wm_win = reply
        .value32()
        .and_then(|mut iter| iter.next())
        .ok_or_else(|| {
            KtileError::EwmhMissing(
                "_NET_SUPPORTING_WM_CHECK absent or empty on root window".to_string(),
            )
        })?;

    // Doppia verifica EWMH sulla finestra child
    let child_reply = conn
        .get_property(
            false,
            wm_win,
            atoms.supporting_wm_check,
            AtomEnum::WINDOW,
            0,
            1,
        )
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    let child_check = child_reply
        .value32()
        .and_then(|mut iter| iter.next())
        .ok_or_else(|| {
            KtileError::EwmhMissing("_NET_SUPPORTING_WM_CHECK invalid on child window".to_string())
        })?;

    if child_check != wm_win {
        return Err(KtileError::EwmhMissing(
            "_NET_SUPPORTING_WM_CHECK mismatch between root and WM window".to_string(),
        ));
    }

    // Lettura _NET_WM_NAME o fallback su WM_NAME
    let name_reply = conn
        .get_property(false, wm_win, atoms.wm_name, AtomEnum::ANY, 0, 1024)
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    if !name_reply.value.is_empty() {
        if let Ok(name) = String::from_utf8(name_reply.value) {
            return Ok(name);
        }
    }

    Ok("Unknown EWMH WM".to_string())
}

/// Legge `_NET_CLIENT_LIST` con fallback su `_NET_CLIENT_LIST_STACKING` (warn).
/// Entrambe assenti → exit 6 (D6).
pub fn client_list(
    conn: &RustConnection,
    root: Window,
    atoms: &EwmhAtoms,
) -> Result<Vec<Window>, KtileError> {
    let reply = conn
        .get_property(false, root, atoms.client_list, AtomEnum::WINDOW, 0, 1024)
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    let list: Vec<Window> = reply
        .value32()
        .map(|iter| iter.collect())
        .unwrap_or_default();
    if !list.is_empty() {
        return Ok(list);
    }

    // Fallback _NET_CLIENT_LIST_STACKING
    let reply_stacking = conn
        .get_property(
            false,
            root,
            atoms.client_list_stacking,
            AtomEnum::WINDOW,
            0,
            1024,
        )
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    let list_stacking: Vec<Window> = reply_stacking
        .value32()
        .map(|iter| iter.collect())
        .unwrap_or_default();

    if !list_stacking.is_empty() {
        eprintln!(
            "ktile: warning: _NET_CLIENT_LIST empty, using _NET_CLIENT_LIST_STACKING fallback"
        );
        return Ok(list_stacking);
    }

    Err(KtileError::EwmhMissing(
        "_NET_CLIENT_LIST and _NET_CLIENT_LIST_STACKING are missing or empty".to_string(),
    ))
}

/// Legge `_NET_ACTIVE_WINDOW` su root.
pub fn get_active_window(
    conn: &RustConnection,
    root: Window,
    atoms: &EwmhAtoms,
) -> Result<Option<Window>, KtileError> {
    let reply = match conn.get_property(false, root, atoms.active_window, AtomEnum::WINDOW, 0, 1) {
        Ok(cookie) => match cookie.reply() {
            Ok(r) => r,
            Err(_) => return Ok(None),
        },
        Err(_) => return Ok(None),
    };

    Ok(reply
        .value32()
        .and_then(|mut i| i.next())
        .filter(|&w| w != 0))
}

/// Legge `_NET_CURRENT_DESKTOP` su root.
pub fn get_current_desktop(
    conn: &RustConnection,
    root: Window,
    atoms: &EwmhAtoms,
) -> Result<u32, KtileError> {
    let reply = conn
        .get_property(false, root, atoms.current_desktop, AtomEnum::CARDINAL, 0, 1)
        .map_err(|e| KtileError::X11(e.to_string()))?
        .reply()
        .map_err(|e| KtileError::X11(e.to_string()))?;

    Ok(reply.value32().and_then(|mut i| i.next()).unwrap_or(0))
}

/// Legge `_NET_WM_WINDOW_TYPE` per una finestra.
pub fn get_wm_window_type(
    conn: &RustConnection,
    atoms: &EwmhAtoms,
    window: Window,
) -> Result<Option<Atom>, KtileError> {
    let reply = match conn.get_property(false, window, atoms.wm_window_type, AtomEnum::ATOM, 0, 1) {
        Ok(cookie) => match cookie.reply() {
            Ok(r) => r,
            Err(_) => return Ok(None),
        },
        Err(_) => return Ok(None),
    };

    Ok(reply.value32().and_then(|mut i| i.next()))
}

/// Legge `_NET_WM_STATE` per una finestra.
pub fn get_wm_state(
    conn: &RustConnection,
    atoms: &EwmhAtoms,
    window: Window,
) -> Result<Vec<Atom>, KtileError> {
    let reply = match conn.get_property(false, window, atoms.wm_state, AtomEnum::ATOM, 0, 32) {
        Ok(cookie) => match cookie.reply() {
            Ok(r) => r,
            Err(_) => return Ok(Vec::new()),
        },
        Err(_) => return Ok(Vec::new()),
    };

    Ok(reply
        .value32()
        .map(|iter| iter.collect())
        .unwrap_or_default())
}

/// Legge `_NET_WM_DESKTOP` per una finestra.
pub fn get_wm_desktop(
    conn: &RustConnection,
    atoms: &EwmhAtoms,
    window: Window,
) -> Result<Option<u32>, KtileError> {
    let reply = match conn.get_property(false, window, atoms.wm_desktop, AtomEnum::CARDINAL, 0, 1) {
        Ok(cookie) => match cookie.reply() {
            Ok(r) => r,
            Err(_) => return Ok(None),
        },
        Err(_) => return Ok(None),
    };

    Ok(reply.value32().and_then(|mut i| i.next()))
}
