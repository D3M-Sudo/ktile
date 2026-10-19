// DRAFT M1-prep — NON compilato, NON testato: toolchain Rust assente sull'host.
// Registry finestre: WindowId → WindowState (Active/Suspended/Excluded/Gone, D8/D9).

/// Identificativo finestra (newtype opaco: layout/ non vede XID grezzi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

/// Stato ciclo di vita (doc 04 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Mappata, nel layout.
    Active,
    /// Unmap/HIDDEN: slot rilasciato, posizione conservata (D8), mai toccata.
    Suspended,
    /// Presente ma mai nel layout (dialoghi, dock, fullscreen, sticky D9…).
    Excluded,
    /// DestroyNotify: rimozione definitiva.
    Gone,
}
