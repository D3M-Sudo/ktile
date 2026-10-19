// Connessione X11 via x11rb: connect, intern atomi EWMH, retry, check estensioni.

use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use crate::errors::KtileError;

/// Atomi EWMH internati al bootstrap (sottoinsieme M1 read-only).
#[derive(Debug, Clone, Copy)]
pub struct EwmhAtoms {
    /// `_NET_SUPPORTING_WM_CHECK` (verifica EWMH, doppia lettura root→WM).
    pub supporting_wm_check: Atom,
    /// `_NET_WM_NAME`.
    pub wm_name: Atom,
    /// `_NET_CLIENT_LIST` (+ fallback `_NET_CLIENT_LIST_STACKING`).
    pub client_list: Atom,
    /// `_NET_CLIENT_LIST_STACKING`.
    pub client_list_stacking: Atom,
    /// `_NET_ACTIVE_WINDOW`.
    pub active_window: Atom,
    /// `_NET_WORKAREA`.
    pub workarea: Atom,
    /// `_NET_WM_DESKTOP`.
    pub wm_desktop: Atom,
    /// `_NET_WM_STATE`.
    pub wm_state: Atom,
    /// `_NET_WM_WINDOW_TYPE`.
    pub wm_window_type: Atom,
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
            wm_desktop: intern(b"_NET_WM_DESKTOP")?,
            wm_state: intern(b"_NET_WM_STATE")?,
            wm_window_type: intern(b"_NET_WM_WINDOW_TYPE")?,
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
