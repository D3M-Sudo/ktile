// DRAFT M1-prep — NON compilato, NON testato: toolchain Rust assente sull'host.
// Errori tipizzati M1 → exit codes (D4: 0 anche senza eleggibili; D6: 6 solo EWMH essenziale assente).

use thiserror::Error;

/// Errori di dominio Ktile (M1: sottoinsieme read-only).
#[derive(Debug, Error)]
pub enum KtileError {
    /// Exit 4: DISPLAY assente o connessione X11 fallita.
    #[error("no X11 session (DISPLAY missing or connection failed)")]
    NoDisplay,
    /// Exit 5: errore X11/EWMH durante l'operazione.
    #[error("X11/EWMH error: {0}")]
    X11(String),
    /// Exit 6: EWMH essenziale assente (no WM check / liste client). Nessuna whitelist nomi (D6).
    #[error("essential EWMH missing: {0}")]
    EwmhMissing(String),
    /// Exit 2: argomenti invalidi.
    #[error("invalid arguments: {0}")]
    Args(String),
}

impl KtileError {
    /// Mappa errori → exit code stabile (doc 04 §8).
    pub fn exit_code(&self) -> u8 {
        match self {
            KtileError::NoDisplay => 4,
            KtileError::X11(_) => 5,
            KtileError::EwmhMissing(_) => 6,
            KtileError::Args(_) => 2,
        }
    }
}
