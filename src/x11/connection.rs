// Connessione X11 via x11rb: connect, intern atomi EWMH, retry, check estensioni.

use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use crate::errors::KtileError;

/// Atomi EWMH internati al bootstrap.
#[derive(Debug, Clone, Copy)]
pub struct EwmhAtoms {
    pub supporting_wm_check: Atom,
    pub wm_name: Atom,
    pub client_list: Atom,
    pub client_list_stacking: Atom,
    pub active_window: Atom,
    pub workarea: Atom,
    pub current_desktop: Atom,
    pub wm_desktop: Atom,
    pub wm_state: Atom,
    pub wm_window_type: Atom,
    pub wm_window_type_normal: Atom,
    pub wm_window_type_dialog: Atom,
    pub wm_window_type_dock: Atom,
    pub wm_state_fullscreen: Atom,
    pub wm_state_hidden: Atom,
}

impl EwmhAtoms {
    pub fn new(conn: &RustConnection) -> Result<Self, KtileError> {
        let intern = |name: &[u8]| -> Result<Atom, KtileError> {
            conn.intern_atom(false, name)
                .map_err(|e| KtileError::X11(e.to_string()))?
                .reply()
                .map_err(|e| KtileError::X11(e.to_string()))
                .map(|r| r.atom)
        };

        Ok(Self {
            supporting_wm_check: intern(b"_NET_SUPPORTING_WM_CHECK")?,
            wm_name: intern(b"_NET_WM_NAME")?,
            client_list: intern(b"_NET_CLIENT_LIST")?,
            client_list_stacking: intern(b"_NET_CLIENT_LIST_STACKING")?,
            active_window: intern(b"_NET_ACTIVE_WINDOW")?,
            workarea: intern(b"_NET_WORKAREA")?,
            current_desktop: intern(b"_NET_CURRENT_DESKTOP")?,
            wm_desktop: intern(b"_NET_WM_DESKTOP")?,
            wm_state: intern(b"_NET_WM_STATE")?,
            wm_window_type: intern(b"_NET_WM_WINDOW_TYPE")?,
            wm_window_type_normal: intern(b"_NET_WM_WINDOW_TYPE_NORMAL")?,
            wm_window_type_dialog: intern(b"_NET_WM_WINDOW_TYPE_DIALOG")?,
            wm_window_type_dock: intern(b"_NET_WM_WINDOW_TYPE_DOCK")?,
            wm_state_fullscreen: intern(b"_NET_WM_STATE_FULLSCREEN")?,
            wm_state_hidden: intern(b"_NET_WM_STATE_HIDDEN")?,
        })
    }
}

/// Stabilisce la connessione X11.
pub fn connect() -> Result<(RustConnection, usize), KtileError> {
    if std::env::var_os("DISPLAY").is_none() {
        return Err(KtileError::NoDisplay);
    }
    let (conn, screen_num) = x11rb::connect(None).map_err(|_| KtileError::NoDisplay)?;
    Ok((conn, screen_num))
}
