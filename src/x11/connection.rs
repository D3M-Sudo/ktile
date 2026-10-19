// DRAFT M1-prep — NON compilato, NON testato: toolchain Rust assente sull'host.
// Connessione X11 via x11rb: connect, intern atomi EWMH, retry, check estensioni.

use crate::errors::KtileError;

/// Atomi EWMH internati al bootstrap (sottoinsieme M1 read-only).
#[derive(Debug, Clone)]
pub struct EwmhAtoms {
    /// `_NET_SUPPORTING_WM_CHECK` (verifica EWMH, doppia lettura root→WM).
    pub supporting_wm_check: u32,
    /// `_NET_WM_NAME`.
    pub wm_name: u32,
    /// `_NET_CLIENT_LIST` (+ fallback `_NET_CLIENT_LIST_STACKING`).
    pub client_list: u32,
    /// `_NET_CLIENT_LIST_STACKING`.
    pub client_list_stacking: u32,
    /// `_NET_ACTIVE_WINDOW`.
    pub active_window: u32,
}

/// Stabilisce la connessione X11 (M1: implementare con x11rb::connect).
pub fn connect() -> Result<(), KtileError> {
    if std::env::var_os("DISPLAY").is_none() {
        return Err(KtileError::NoDisplay);
    }
    // DRAFT: x11rb::connect(None) → intern atoms → Ok
    Ok(())
}
