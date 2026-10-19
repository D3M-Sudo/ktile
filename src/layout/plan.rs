// Piano di mosse geometriche e calcolo diff (idempotenza).

use crate::layout::geometry::Rect;
use crate::state::registry::WindowId;

/// Mossa geometrica singola.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowMove {
    pub window_id: WindowId,
    pub target_rect: Rect,
}

/// Piano delle mosse da eseguire.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    pub moves: Vec<WindowMove>,
}

impl Plan {
    pub fn new(moves: Vec<WindowMove>) -> Self {
        Self { moves }
    }

    /// True se non vi sono mosse da eseguire (idempotente).
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Genera il piano contenente solo le finestre la cui geometria differisce dallo stato atteso.
    pub fn diff(current: &[(WindowId, Rect)], expected: &[(WindowId, Rect)]) -> Self {
        let mut moves = Vec::new();
        for &(exp_id, exp_rect) in expected {
            let needs_move = match current.iter().find(|&&(curr_id, _)| curr_id == exp_id) {
                Some(&(_, curr_rect)) => curr_rect != exp_rect,
                None => true,
            };
            if needs_move {
                moves.push(WindowMove {
                    window_id: exp_id,
                    target_rect: exp_rect,
                });
            }
        }
        Self { moves }
    }
}
