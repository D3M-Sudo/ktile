// M3: layout Master-Stack — doc 04 §5.
// Master a sinistra con larghezza mfact (clamp [0.25, 0.75]), stack a destra diviso in righe
// di altezza uguale (resto distribuito alle prime righe). Master e stack sono liste ordinate:
// la rimozione del master promuove la prima finestra dello stack, senza riequilibrio.
// Nota: anche con 1 finestra il master resta a mfact (coerente con la fixture M2 `master-n1`);
// la coincidenza con Dwindle a n=1 del doc 04 §5 è documentata come divergenza nota.

use crate::layout::geometry::{Gaps, Rect};
use crate::layout::tree::ResumeOutcome;
use crate::state::registry::WindowId;

/// Soglia inferiore/superiore di mfact (doc 04 §5, come millesimi).
const MFACT_MIN_MILLI: u32 = 250;
const MFACT_MAX_MILLI: u32 = 750;

/// Dove finisce una finestra appena aperta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewWindowPlacement {
    /// Diventa il nuovo master, il vecchio master scende in testa allo stack (default).
    #[default]
    Master,
    /// Va in cima allo stack senza toccare il master.
    StackTop,
}

/// Configurazione Master-Stack (default mfact 0.55, doc 04 §5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MasterStackConfig {
    /// Frazione di larghezza del master (default 0.55), clamp [0.25, 0.75].
    pub mfact: f64,
    /// Collocazione delle finestre nuove.
    pub new_placement: NewWindowPlacement,
}

impl Default for MasterStackConfig {
    fn default() -> Self {
        Self {
            mfact: 0.55,
            new_placement: NewWindowPlacement::Master,
        }
    }
}

impl MasterStackConfig {
    /// mfact in millesimi clampato [250, 750] (convenzione milli-unità del progetto).
    pub fn mfact_milli(&self) -> u32 {
        let milli = (self.mfact * 1000.0).round() as u32;
        milli.clamp(MFACT_MIN_MILLI, MFACT_MAX_MILLI)
    }
}

/// Slot sospeso Master-Stack (D8): ricorda la posizione originaria e la lunghezza della lista
/// al momento della sospensione, per distinguere uno slot ancora identificabile da uno no.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MsParked {
    /// Finestra sospesa.
    window: WindowId,
    /// Posizione originaria (0 = era il master).
    index: usize,
    /// Lunghezza della lista (master+stack) subito dopo la sospensione.
    len_after: usize,
}

/// Layout Master-Stack: master + stack ordinati, MRU per la selezione (doc 04 §5).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MasterStack {
    master: Option<WindowId>,
    stack: Vec<WindowId>,
    mru: Vec<WindowId>,
    parked: Vec<MsParked>,
    config: MasterStackConfig,
}

impl MasterStack {
    /// Layout vuoto con la data config.
    pub fn new(config: MasterStackConfig) -> Self {
        Self {
            master: None,
            stack: Vec::new(),
            mru: Vec::new(),
            parked: Vec::new(),
            config,
        }
    }

    /// Config corrente.
    pub fn config(&self) -> &MasterStackConfig {
        &self.config
    }

    /// Finestra master, se presente.
    pub fn master(&self) -> Option<WindowId> {
        self.master
    }

    /// Finestre dello stack, in ordine (testa = primo dopo il master).
    pub fn stack(&self) -> &[WindowId] {
        &self.stack
    }

    /// Finestre in ordine DFS equivalente: master prima, poi stack.
    pub fn windows_dfs(&self) -> Vec<WindowId> {
        let mut out = Vec::with_capacity(self.stack.len() + 1);
        if let Some(m) = self.master {
            out.push(m);
        }
        out.extend_from_slice(&self.stack);
        out
    }

    /// True se la finestra è in tiling (master o stack).
    pub fn contains(&self, window: WindowId) -> bool {
        self.master == Some(window) || self.stack.contains(&window)
    }

    /// Registra `window` come uso più recente (testa MRU).
    pub fn mark_used(&mut self, window: WindowId) {
        self.mru.retain(|&w| w != window);
        self.mru.insert(0, window);
    }

    /// Finestra più di recente usata, se ancora in tiling.
    pub fn mru_head(&self) -> Option<WindowId> {
        self.mru.iter().copied().find(|&w| self.contains(w))
    }

    /// Selezione D3: attiva se in tiling, altrimenti MRU, altrimenti il master.
    pub fn leaf_for(&self, active: Option<WindowId>, mru: Option<WindowId>) -> Option<WindowId> {
        if let Some(a) = active {
            if self.contains(a) {
                return Some(a);
            }
        }
        if let Some(m) = mru {
            if self.contains(m) {
                return Some(m);
            }
        }
        self.mru_head().or(self.master)
    }

    /// Inserisce `window` secondo `new_placement` (doc 04 §5). Restituisce false se già presente.
    pub fn insert(&mut self, window: WindowId) -> bool {
        if self.contains(window) {
            return false;
        }
        self.parked.retain(|p| p.window != window);
        match self.config.new_placement {
            NewWindowPlacement::Master => {
                // Il vecchio master (se c'era) diventa la nuova testa dello stack.
                if let Some(old) = self.master.replace(window) {
                    self.stack.insert(0, old);
                }
            }
            NewWindowPlacement::StackTop => {
                if self.master.is_none() {
                    self.master = Some(window);
                } else {
                    self.stack.insert(0, window);
                }
            }
        }
        self.mark_used(window);
        true
    }

    /// Chiude/distrugge `window`: promozione del primo dello stack al posto del master.
    pub fn close(&mut self, window: WindowId) -> bool {
        self.parked.retain(|p| p.window != window);
        self.mru.retain(|&w| w != window);
        if self.master == Some(window) {
            self.master = self.stack.first().copied();
            if self.master.is_some() {
                self.stack.remove(0);
            }
            return true;
        }
        let before = self.stack.len();
        self.stack.retain(|&w| w != window);
        before != self.stack.len()
    }

    /// Sospende `window` (D8): esce dal tiling con promozione, posizione registrata.
    pub fn suspend(&mut self, window: WindowId) -> bool {
        let index = self.windows_dfs().iter().position(|&w| w == window);
        let closed = self.close(window);
        if let Some(index) = index {
            let len_after = self.windows_dfs().len();
            self.parked.push(MsParked {
                window,
                index,
                len_after,
            });
        }
        closed
    }

    /// Rientro D8: se la lista è invariata dalla sospensione lo slot è ancora identificabile e
    /// la finestra torna alla posizione originaria; altrimenti segue `new_placement`.
    pub fn resume(
        &mut self,
        window: WindowId,
        _active: Option<WindowId>,
        _mru: Option<WindowId>,
    ) -> ResumeOutcome {
        if self.contains(window) {
            self.mark_used(window);
            return ResumeOutcome::InPlace;
        }
        if let Some(idx) = self.parked.iter().position(|p| p.window == window) {
            let MsParked {
                window,
                index,
                len_after,
            } = self.parked.remove(idx);
            if self.windows_dfs().len() == len_after {
                if index == 0 {
                    if let Some(old) = self.master.replace(window) {
                        self.stack.insert(0, old);
                    }
                } else {
                    let pos = index.saturating_sub(1).min(self.stack.len());
                    self.stack.insert(pos, window);
                }
                self.mark_used(window);
                return ResumeOutcome::InPlace;
            }
        }
        if self.insert(window) {
            ResumeOutcome::NewLeaf
        } else {
            ResumeOutcome::InPlace
        }
    }
}

impl MasterStack {
    /// Rectangolo per ciascuna finestra, in ordine master→stack.
    /// Master: larghezza mfact dell'area utile; stack: colonna a destra divisa in righe
    /// di altezza uguale (resto distribuito alle prime righe). Gap esterno una volta.
    pub fn compute_rects(&self, area: Rect, gaps: &Gaps) -> Vec<(WindowId, Rect)> {
        let windows = self.windows_dfs();
        if windows.is_empty() {
            return Vec::new();
        }
        let usable = area.apply_outer_gap(gaps.outer);
        let usable_w = usable.width.saturating_sub(gaps.inner);
        let mfact = (self.config.mfact_milli() as f64) / 1000.0;
        let master_w = ((usable_w as f64) * mfact).floor() as u32;

        let master_rect = Rect {
            x: usable.x,
            y: usable.y,
            width: master_w,
            height: usable.height,
        };

        // Con una sola finestra il master occupa la larghezza mfact (coerente con M2 `master-n1`).
        if windows.len() == 1 {
            return vec![(windows[0], master_rect)];
        }

        let stack_w = usable_w.saturating_sub(master_w);
        let stack_count = (windows.len() - 1) as u32;
        let stack_x = usable
            .x
            .saturating_add((master_w.saturating_add(gaps.inner)) as i32);
        let total_inner = gaps.inner.saturating_mul(stack_count.saturating_sub(1));
        let usable_h = usable.height.saturating_sub(total_inner);
        let base_h = usable_h / stack_count;
        let rem = usable_h % stack_count;

        let mut out = Vec::with_capacity(windows.len());
        out.push((windows[0], master_rect));

        let mut current_y = usable.y;
        for (i, &w) in windows[1..].iter().enumerate() {
            let h_i = base_h + if (i as u32) < rem { 1 } else { 0 };
            out.push((
                w,
                Rect {
                    x: stack_x,
                    y: current_y,
                    width: stack_w,
                    height: h_i,
                },
            ));
            current_y = current_y.saturating_add((h_i.saturating_add(gaps.inner)) as i32);
        }
        out
    }
}
