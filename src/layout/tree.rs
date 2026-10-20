// M3: albero binario persistente per (desktop, monitor) — doc 04 §4.
// Ogni split memorizza `dir` e `ratio_milli` nel nodo (preserve_split): nessun riequilibrio,
// le strutture esistenti non vengono mai riscritte. Modello puro, nessuna dipendenza da X11.
//
// Rimozione/promozione: la foglia chiusa diventa vuota; la cessione della regione al fratello
// avviene a livello di geometria (compute_rects), dove un sottoalbero senza finestre cede
// l'intera regione al fratello — risultato rettangolare equivalente alla promozione strutturale.
// La struttura resta intatta così lo slot resta identificabile per D8.

use crate::state::registry::WindowId;

/// Direzione di uno split, persistente nel nodo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDir {
    /// Divisione sinistra/destra: `first` riceve la parte sinistra.
    Horizontal,
    /// Divisione sopra/sotto: `first` riceve la parte superiore.
    Vertical,
}

impl SplitDir {
    /// Direzione opposta (usata da `split_mode = alternating`).
    pub fn flip(self) -> Self {
        match self {
            SplitDir::Horizontal => SplitDir::Vertical,
            SplitDir::Vertical => SplitDir::Horizontal,
        }
    }
}

/// Nodo dell'albero persistente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Foglia: finestra assegnata (None = slot vuoto, vedi D8).
    Leaf {
        /// Finestra nella foglia, se presente.
        window: Option<WindowId>,
    },
    /// Split: `first` la prima metà, `second` la seconda (dove entra la nuova finestra).
    Split {
        /// Direzione persistente dello split.
        dir: SplitDir,
        /// Rapporto in millesimi della prima metà (500 = metà), clamp [250, 750].
        ratio_milli: u32,
        /// Sottolbero `first` (contenuto preesistente del target).
        first: Box<Node>,
        /// Sottolbero `second` (nuova finestra, doc 04 §4).
        second: Box<Node>,
    },
}

/// Slot sospeso (D8): posizione originaria registrata alla sospensione.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParkedSlot {
    /// Finestra sospesa.
    pub window: WindowId,
    /// Percorso dalla radice (false = first, true = second) alla foglia occupata.
    pub path: Vec<bool>,
}

/// Esito del rientro di una finestra sospesa (D8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeOutcome {
    /// Slot ancora identificabile: rientro nella posizione originaria.
    InPlace,
    /// Slot non identificabile (occupato o mai registrato): nuova foglia via MRU/attiva.
    NewLeaf,
}

/// Albero persistente di un (desktop, monitor). Parte vuoto: la prima finestra è foglia intera.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tree {
    root: Option<Node>,
    /// Finestre ordinate per uso recente (testa = più recente) per la selezione D3.
    mru: Vec<WindowId>,
    /// Slot sospesi in attesa di rientro (D8).
    parked: Vec<ParkedSlot>,
}

/// Conta le finestre assegnate in un sottolbero.
pub(crate) fn count_windows(node: &Node) -> usize {
    match node {
        Node::Leaf { window } => usize::from(window.is_some()),
        Node::Split { first, second, .. } => count_windows(first) + count_windows(second),
    }
}

/// Raccoglie le finestre in ordine DFS (`first` prima di `second`).
fn collect_dfs(node: &Node, out: &mut Vec<WindowId>) {
    match node {
        Node::Leaf { window } => {
            if let Some(w) = window {
                out.push(*w);
            }
        }
        Node::Split { first, second, .. } => {
            collect_dfs(first, out);
            collect_dfs(second, out);
        }
    }
}

/// Sostituisce la foglia `target` con uno split: preesistente in `first`, nuova in `second`.
fn insert_into(
    node: &mut Node,
    new_window: WindowId,
    target: WindowId,
    dir: SplitDir,
    ratio_milli: u32,
) -> bool {
    match node {
        Node::Leaf { window } => {
            if *window == Some(target) {
                *node = Node::Split {
                    dir,
                    ratio_milli,
                    first: Box::new(Node::Leaf { window: *window }),
                    second: Box::new(Node::Leaf {
                        window: Some(new_window),
                    }),
                };
                true
            } else {
                false
            }
        }
        Node::Split { first, second, .. } => {
            insert_into(first, new_window, target, dir, ratio_milli)
                || insert_into(second, new_window, target, dir, ratio_milli)
        }
    }
}

/// Svuota la foglia che contiene `target`. Restituisce true se `target` era presente.
fn empty_leaf(node: &mut Node, target: WindowId) -> bool {
    match node {
        Node::Leaf { window } => {
            if *window == Some(target) {
                *window = None;
                true
            } else {
                false
            }
        }
        Node::Split { first, second, .. } => {
            empty_leaf(first, target) || empty_leaf(second, target)
        }
    }
}

/// Percorso dalla radice alla foglia di `target` (false = first, true = second).
fn path_of(node: &Node, target: WindowId, prefix: &mut Vec<bool>) -> Option<Vec<bool>> {
    match node {
        Node::Leaf { window } => (*window == Some(target)).then(|| prefix.clone()),
        Node::Split { first, second, .. } => {
            prefix.push(false);
            if let Some(p) = path_of(first, target, prefix) {
                return Some(p);
            }
            prefix.pop();
            prefix.push(true);
            let found = path_of(second, target, prefix);
            if found.is_none() {
                prefix.pop();
            }
            found
        }
    }
}

/// Naviga fino al nodo indicato dal percorso; None se il percorso è più lungo del ramo.
fn node_at_mut<'a>(node: &'a mut Node, path: &[bool]) -> Option<&'a mut Node> {
    let mut current = node;
    for &go_second in path {
        match current {
            Node::Split { first, second, .. } => current = if go_second { second } else { first },
            Node::Leaf { .. } => return None,
        }
    }
    Some(current)
}

/// Versione immutabile di `node_at_mut`.
fn node_at<'a>(node: &'a Node, path: &[bool]) -> Option<&'a Node> {
    let mut current = node;
    for &go_second in path {
        match current {
            Node::Split { first, second, .. } => current = if go_second { second } else { first },
            Node::Leaf { .. } => return None,
        }
    }
    Some(current)
}

impl Tree {
    /// Albero vuoto.
    pub fn new() -> Self {
        Self::default()
    }

    /// Radice dell'albero (per la computazione geometrica).
    pub fn root(&self) -> Option<&Node> {
        self.root.as_ref()
    }

    /// Numero di finestre assegnate (le foglie vuote non contano).
    pub fn len(&self) -> usize {
        self.root.as_ref().map_or(0, count_windows)
    }

    /// True se non contiene finestre.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Finestre in ordine DFS: ordine stabile D3.
    pub fn windows_dfs(&self) -> Vec<WindowId> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            collect_dfs(root, &mut out);
        }
        out
    }

    /// True se la finestra è assegnata a una foglia dell'albero.
    pub fn contains(&self, window: WindowId) -> bool {
        self.windows_dfs().contains(&window)
    }

    /// Direzione dello split che contiene la foglia di `window` (genitore).
    /// Serve a `split_mode = alternating` (direzione opposta al padre).
    /// None se `window` non c'è o è la foglia radice.
    pub fn parent_dir_of(&self, window: WindowId) -> Option<SplitDir> {
        let root = self.root.as_ref()?;
        let path = path_of(root, window, &mut Vec::new())?;
        if path.is_empty() {
            return None;
        }
        match node_at(root, &path[..path.len() - 1])? {
            Node::Split { dir, .. } => Some(*dir),
            Node::Leaf { .. } => None,
        }
    }

    /// Registra `window` come uso più recente (testa MRU).
    pub fn mark_used(&mut self, window: WindowId) {
        self.mru.retain(|&w| w != window);
        self.mru.insert(0, window);
    }

    /// Finestra più di recente usata, se ancora presente nell'albero.
    pub fn mru_head(&self) -> Option<WindowId> {
        self.mru.iter().copied().find(|&w| self.contains(w))
    }

    /// Selezione foglia D3: attiva se è in tiling, altrimenti MRU, altrimenti la prima DFS.
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
        self.mru_head()
            .or_else(|| self.windows_dfs().first().copied())
    }

    /// Inserisce `new_window` dividendo la foglia di `target` (nuova finestra in `second`).
    /// Albero vuoto → prima foglia intera. Restituisce false se `target` non è nell'albero.
    pub fn insert_split(
        &mut self,
        new_window: WindowId,
        target: WindowId,
        dir: SplitDir,
        ratio_milli: u32,
    ) -> bool {
        let ratio_milli = ratio_milli.clamp(250, 750);
        match self.root.take() {
            None => {
                self.root = Some(Node::Leaf {
                    window: Some(new_window),
                });
                self.mark_used(new_window);
                true
            }
            Some(mut root) => {
                let inserted = insert_into(&mut root, new_window, target, dir, ratio_milli);
                self.root = Some(root);
                if inserted {
                    self.mark_used(new_window);
                }
                inserted
            }
        }
    }

    /// Chiude/distrugge `window`: foglia svuotata (la regione passa al fratello in geometria),
    /// rimossa da MRU e da eventuali slot sospesi.
    pub fn close(&mut self, window: WindowId) -> bool {
        self.parked.retain(|p| p.window != window);
        self.mru.retain(|&w| w != window);
        match self.root.as_mut() {
            Some(root) => empty_leaf(root, window),
            None => false,
        }
    }

    /// Sospende `window` (D8): foglia svuotata e posizione originaria registrata.
    pub fn suspend(&mut self, window: WindowId) -> bool {
        let path = self
            .root
            .as_ref()
            .and_then(|r| path_of(r, window, &mut Vec::new()));
        let closed = self.close(window);
        if let Some(path) = path {
            self.parked.push(ParkedSlot { window, path });
        }
        closed
    }

    /// Rientro D8: slot identificabile → posizione originaria, altrimenti nuova foglia
    /// (split sulla foglia scelta da `leaf_for`). `dir`/`ratio_milli` valgono solo nel fallback.
    pub fn resume(
        &mut self,
        window: WindowId,
        active: Option<WindowId>,
        mru: Option<WindowId>,
        dir: SplitDir,
        ratio_milli: u32,
    ) -> ResumeOutcome {
        if self.contains(window) {
            self.mark_used(window);
            return ResumeOutcome::InPlace;
        }
        if let Some(idx) = self.parked.iter().position(|p| p.window == window) {
            let path = self.parked.remove(idx).path;
            let slot_free = match self.root.as_mut().and_then(|r| node_at_mut(r, &path)) {
                Some(Node::Leaf { window: slot }) if slot.is_none() => {
                    *slot = Some(window);
                    true
                }
                _ => false,
            };
            if slot_free {
                self.mark_used(window);
                return ResumeOutcome::InPlace;
            }
        }
        // Fallback: nuova foglia sulla target scelta (MRU/attiva); albero vuoto → prima foglia.
        let target = self.leaf_for(active, mru);
        match target {
            Some(t) => {
                self.insert_split(window, t, dir, ratio_milli);
                ResumeOutcome::NewLeaf
            }
            None => {
                self.root = Some(Node::Leaf {
                    window: Some(window),
                });
                self.mark_used(window);
                ResumeOutcome::NewLeaf
            }
        }
    }
}
