// M3: layout Dwindle sul tree persistente — doc 04 §4.
// Pipeline: config clamp → selezione foglia (D3: attiva, altrimenti MRU) → direzione
// (longest-side o alternanza rispetto al padre) → inserimento in `second` → compute_rects.
// Fallback MIN_SIDE (50px): se la nuova finestra non starebbe in nessuna foglia con un lato
// >= 50px, tenta la foglia più capiente; altrimenti resta floating (mai pani, solo esito).

use crate::layout::geometry::{Gaps, Rect, SplitMode};
use crate::layout::tree::{count_windows, Node, ResumeOutcome, SplitDir, Tree};
use crate::state::registry::WindowId;

/// Lato minimo sotto il quale una finestra non viene inserita nel tiling (doc 04 §5).
pub const MIN_SIDE: u32 = 50;

/// Finestra usata come target dello split quando l'attiva non è in tiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitTarget {
    /// Attiva se è in tiling, altrimenti MRU (default, doc 04 §4).
    #[default]
    Active,
    /// Sempre la MRU (se presente).
    Mru,
}

/// Configurazione Dwindle (default 0.50).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DwindleConfig {
    /// Direzione di split: lungo il lato maggiore o alternanza.
    pub split_mode: SplitMode,
    /// Rapporto della prima metà (default 0.50), clamp [0.25, 0.75].
    pub split_ratio: f64,
    /// Chi fa da target dello split.
    pub split_target: SplitTarget,
}

impl Default for DwindleConfig {
    fn default() -> Self {
        Self {
            split_mode: SplitMode::LongestSide,
            split_ratio: 0.50,
            split_target: SplitTarget::Active,
        }
    }
}

impl DwindleConfig {
    /// Rapporto in millesimi (es. 0.55 → 550). Il clamp [250, 750] è applicato dal tree all'inserimento.
    pub fn ratio_milli(&self) -> u32 {
        (self.split_ratio * 1000.0).round() as u32
    }
}

/// Layout Dwindle: albero persistente + config (doc 04 §4).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dwindle {
    tree: Tree,
    config: DwindleConfig,
}

/// Lato minore di un rettangolo (per il confronto con MIN_SIDE).
fn min_side(rect: &Rect) -> u32 {
    rect.width.min(rect.height)
}

/// Direzione di split per la foglia `target` secondo la config (doc 04 §4).
fn choose_dir(
    config: &DwindleConfig,
    tree: &Tree,
    target: WindowId,
    target_rect: Rect,
) -> SplitDir {
    let longest = if target_rect.width >= target_rect.height {
        SplitDir::Horizontal
    } else {
        SplitDir::Vertical
    };
    match config.split_mode {
        SplitMode::LongestSide => longest,
        // Alternanza: direzione opposta al padre; la radice cade sul longest-side.
        SplitMode::Alternating => tree.parent_dir_of(target).map_or(longest, SplitDir::flip),
    }
}

/// Rectangolo che la nuova finestra ottenerebbe dividendo `target_rect` in `dir`.
fn prospective_second(rect: &Rect, dir: SplitDir, ratio: f64, inner: u32) -> Rect {
    match dir {
        SplitDir::Horizontal => rect.split_horizontal(ratio, inner).1,
        SplitDir::Vertical => rect.split_vertical(ratio, inner).1,
    }
}

impl Dwindle {
    /// Layout vuoto con la data config.
    pub fn new(config: DwindleConfig) -> Self {
        Self {
            tree: Tree::new(),
            config,
        }
    }

    /// Config corrente.
    pub fn config(&self) -> &DwindleConfig {
        &self.config
    }

    /// Albero persistente sottostante (accesso in sola lettura).
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    /// Inserisce `window` secondo la pipeline Dwindle (doc 04 §4) dati attiva/MRU e area di lavoro.
    /// Il fallback MIN_SIDE prova la foglia più capiente; se anche quella è troppo stretta la
    /// finestra resta floating (`InsertOutcome::Floating`), senza panic.
    pub fn insert(
        &mut self,
        window: WindowId,
        active: Option<WindowId>,
        mru: Option<WindowId>,
        area: Rect,
        gaps: &Gaps,
    ) -> InsertOutcome {
        let ratio = self.config.split_ratio.clamp(0.25, 0.75);
        let ratio_milli = self.config.ratio_milli();

        // Prima finestra: foglia intera.
        if self.tree.is_empty() {
            self.tree
                .insert_split(window, window, SplitDir::Horizontal, ratio_milli);
            return InsertOutcome::Tiled;
        }

        let target = match self.config.split_target {
            SplitTarget::Active => self.tree.leaf_for(active, mru),
            SplitTarget::Mru => self
                .tree
                .mru_head()
                .or_else(|| self.tree.leaf_for(active, mru)),
        };
        let Some(target) = target else {
            return InsertOutcome::Floating;
        };

        // Candidate: prima il target scelto, poi le altre foglie per capienza decrescente.
        let rects = self.compute_rects(area, gaps);
        let target_rect = rects.iter().find(|(w, _)| *w == target).map(|(_, r)| *r);

        if let Some(target_rect) = target_rect {
            let dir = choose_dir(&self.config, &self.tree, target, target_rect);
            let second = prospective_second(&target_rect, dir, ratio, gaps.inner);
            if min_side(&second) >= MIN_SIDE {
                self.tree.insert_split(window, target, dir, ratio_milli);
                return InsertOutcome::Tiled;
            }
        }

        // Fallback MIN_SIDE: foglia più capiente che produca almeno MIN_SIDE.
        let mut by_capacity: Vec<(WindowId, Rect)> = rects;
        by_capacity.sort_by_key(|(_, r)| std::cmp::Reverse(r.width * r.height));
        for (cand, cand_rect) in by_capacity {
            if cand == target {
                continue;
            }
            let dir = choose_dir(&self.config, &self.tree, cand, cand_rect);
            let second = prospective_second(&cand_rect, dir, ratio, gaps.inner);
            if min_side(&second) >= MIN_SIDE {
                self.tree.insert_split(window, cand, dir, ratio_milli);
                return InsertOutcome::Tiled;
            }
        }
        InsertOutcome::Floating
    }
}

/// Raccoglie i rect delle foglie assegnate, cedendo la regione di un ramo vuoto al fratello
/// (≡ promozione strutturale, doc 04 §4).
fn collect_rects(node: &Node, area: Rect, gaps: &Gaps, out: &mut Vec<(WindowId, Rect)>) {
    match node {
        Node::Leaf { window } => {
            if let Some(w) = window {
                out.push((*w, area));
            }
        }
        Node::Split {
            dir,
            ratio_milli,
            first,
            second,
        } => {
            let cf = count_windows(first);
            let cs = count_windows(second);
            match (cf, cs) {
                (0, 0) => return,
                // Ramo vuoto: il fratello prende l'intera regione (promozione).
                (0, _) => {
                    collect_rects(second, area, gaps, out);
                    return;
                }
                (_, 0) => {
                    collect_rects(first, area, gaps, out);
                    return;
                }
                _ => {}
            }
            let ratio = (*ratio_milli as f64) / 1000.0;
            let (r_first, r_second) = match dir {
                SplitDir::Horizontal => area.split_horizontal(ratio, gaps.inner),
                SplitDir::Vertical => area.split_vertical(ratio, gaps.inner),
            };
            collect_rects(first, r_first, gaps, out);
            collect_rects(second, r_second, gaps, out);
        }
    }
}

impl Dwindle {
    /// Rectangolo per ciascuna finestra in tiling, in ordine DFS. Area di lavoro meno gap esterno.
    pub fn compute_rects(&self, area: Rect, gaps: &Gaps) -> Vec<(WindowId, Rect)> {
        let mut out = Vec::new();
        if let Some(root) = self.tree.root() {
            let usable = area.apply_outer_gap(gaps.outer);
            collect_rects(root, usable, gaps, &mut out);
        }
        out
    }

    /// Chiude/distrugge `window` (foglia svuotata, regione promossa in geometria).
    pub fn close(&mut self, window: WindowId) -> bool {
        self.tree.close(window)
    }

    /// Sospende `window` preservando lo slot D8.
    pub fn suspend(&mut self, window: WindowId) -> bool {
        self.tree.suspend(window)
    }

    /// Rientro D8: posizione originaria se lo slot è identificabile, altrimenti MRU/attiva.
    pub fn resume(
        &mut self,
        window: WindowId,
        active: Option<WindowId>,
        mru: Option<WindowId>,
    ) -> ResumeOutcome {
        let dir = if self.config.split_mode == SplitMode::Alternating {
            self.tree
                .parent_dir_of(active.unwrap_or(window))
                .map_or(SplitDir::Horizontal, SplitDir::flip)
        } else {
            SplitDir::Horizontal
        };
        self.tree
            .resume(window, active, mru, dir, self.config.ratio_milli())
    }
}

/// Esito di un'inserimento Dwindle (nessuna dipendenza dal logger: il chiamante decide).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    /// Finestra inserita nel tiling.
    Tiled,
    /// Rientro D8 nella posizione originaria.
    ResumedInPlace,
    /// Nessuna foglia accogliente (sotto MIN_SIDE): resta floating.
    Floating,
}
