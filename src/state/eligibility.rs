// Policy pura di eleggibilità delle finestre (doc 04 §3).

use crate::state::registry::WindowId;

/// Tipo di finestra EWMH/ICCCM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Normal,
    Dialog,
    Dock,
    Toolbar,
    Menu,
    Tooltip,
    Splash,
    Notification,
    Dropdown,
    Popup,
    Utility,
    Unknown,
}

/// Stati atomicamente tracciati da _NET_WM_STATE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowStateAtom {
    Fullscreen,
    Hidden,
    SkipTaskbar,
    SkipPager,
    Modal,
    Above,
    Below,
    MaximizedHorz,
    MaximizedVert,
    Other,
}

/// Metadati raccolti per la decisione di eleggibilità.
#[derive(Debug, Clone)]
pub struct WindowMeta {
    pub id: WindowId,
    pub is_mapped: bool,
    pub window_type: Option<WindowType>,
    pub transient_for: Option<WindowId>,
    pub states: Vec<WindowStateAtom>,
    pub desktop: u32,
    pub class: Option<String>,
    pub instance: Option<String>,
    pub title: Option<String>,
}

/// Configurazione di esclusione da TOML.
#[derive(Debug, Clone, Default)]
pub struct ExcludeConfig {
    pub classes: Vec<String>,
    pub instances: Vec<String>,
}

/// Determina se una finestra è eleggibile per il tiling automatico (doc 04 §3).
pub fn is_eligible(meta: &WindowMeta, current_desktop: u32, config: &ExcludeConfig) -> bool {
    if !meta.is_mapped {
        return false;
    }

    // 1. Tipo: solo NORMAL o assente
    if let Some(ref wtype) = meta.window_type {
        if *wtype != WindowType::Normal {
            return false;
        }
    }

    // 2. Transient: dev'essere assente
    if meta.transient_for.is_some() {
        return false;
    }

    // 3. Stati vietati: Fullscreen, Hidden, SkipTaskbar/Pager, Modal, Above/Below
    for state in &meta.states {
        match state {
            WindowStateAtom::Fullscreen
            | WindowStateAtom::Hidden
            | WindowStateAtom::SkipTaskbar
            | WindowStateAtom::SkipPager
            | WindowStateAtom::Modal
            | WindowStateAtom::Above
            | WindowStateAtom::Below => return false,
            _ => {}
        }
    }

    // 4. Desktop: dev'essere il corrente. Sticky (0xFFFFFFFF) escluso in v1 (D9)
    if meta.desktop == 0xFFFFFFFF || meta.desktop != current_desktop {
        return false;
    }

    // 5. Configurazione esclusioni
    if let Some(ref class) = meta.class {
        if config.classes.iter().any(|c| c.eq_ignore_ascii_case(class)) {
            return false;
        }
    }

    if let Some(ref instance) = meta.instance {
        if config
            .instances
            .iter()
            .any(|i| i.eq_ignore_ascii_case(instance))
        {
            return false;
        }
    }

    true
}
