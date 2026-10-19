// Selezione eventi root + traduzione in DomainEvent. M1: log Map/Unmap/Destroy/PropertyNotify 30s.

/// Eventi di dominio (lo stato reagisce a questi, non all'evento X11 grezzo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainEvent {
    /// Finestra mappata (candidata Active).
    WindowMapped(u32),
    /// Finestra smappata (candidata Suspended, D8 — NON assunta chiusa).
    WindowUnmapped(u32),
    /// Finestra distrutta (Gone definitiva — unica rimozione certa).
    WindowDestroyed(u32),
    /// Proprietà cambiata su finestra (rileggere sempre il valore).
    PropertiesChanged(u32),
    /// Focus cambiato (indicazione `_NET_ACTIVE_WINDOW`).
    FocusChanged,
}

/// Seleziona le maschere eventi su root + finestre tracked.
pub fn select_root_events() {
    // DRAFT: SubstructureNotify | PropertyChange su root.
}

/// Osserva gli eventi per 30s in modalità read-only, log su stderr.
pub fn observe_30s() {
    // DRAFT: poll loop con timeout, nessuna richiesta di modifica geometrie.
}
