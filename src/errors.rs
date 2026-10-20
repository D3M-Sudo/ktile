// Errori tipizzati → exit codes (doc 04 §8).

use thiserror::Error;

/// Errori di dominio Ktile.
#[derive(Debug, Error)]
pub enum KtileError {
    /// Exit 2: argomenti CLI invalidi.
    #[error("invalid arguments: {0}")]
    Args(String),
    /// Exit 3: configurazione invalida.
    #[error("invalid config: {0}")]
    ConfigInvalid(String),
    /// Exit 4: DISPLAY assente o connessione X11 fallita.
    #[error("no X11 session (DISPLAY missing or connection failed)")]
    NoDisplay,
    /// Exit 5: errore X11/EWMH durante l'operazione.
    #[error("X11/EWMH error: {0}")]
    X11(String),
    /// Exit 6: EWMH essenziale assente (no WM check / liste client).
    #[error("essential EWMH missing: {0}")]
    EwmhMissing(String),
}

impl KtileError {
    /// Mappa errori → exit code stabile (doc 04 §8).
    pub fn exit_code(&self) -> u8 {
        match self {
            KtileError::Args(_) => 2,
            KtileError::ConfigInvalid(_) => 3,
            KtileError::NoDisplay => 4,
            KtileError::X11(_) => 5,
            KtileError::EwmhMissing(_) => 6,
        }
    }
}
