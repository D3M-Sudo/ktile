// DRAFT M1-prep — NON compilato, NON testato: toolchain Rust assente sull'host.
// Lettura proprietà EWMH (read-only): doppia verifica WM check, liste client, active window.

use crate::errors::KtileError;

/// Verifica EWMH essenziale: lettura `_NET_SUPPORTING_WM_CHECK` su root → XID,
/// rilettura sulla finestra WM + `_NET_WM_NAME`. Assente → exit 6 (D6).
pub fn verify_wm() -> Result<String, KtileError> {
    // DRAFT: get_property(root, SUPPORTING_WM_CHECK) → wid → get_property(wid, WM_NAME)
    // Verificato manualmente M0: 0x2600006 "Mutter (Muffin)".
    Err(KtileError::EwmhMissing(
        "DRAFT: implementare lettura _NET_SUPPORTING_WM_CHECK".to_string(),
    ))
}

/// Legge `_NET_CLIENT_LIST` con fallback su `_NET_CLIENT_LIST_STACKING` (warn).
/// Entrambe assenti → exit 6 (D6).
pub fn client_list() -> Result<Vec<u32>, KtileError> {
    // DRAFT. Verificato manualmente M0: [0x2a00021, 0x5200017, 0x5400004].
    Ok(vec![])
}
