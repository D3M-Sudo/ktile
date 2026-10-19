// Geometria pura: Rect, Gaps, split helpers, clamp e calcolo layout.

/// Rettangolo geometrico (i32/u32 con conversioni sature e sicure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Gaps interno ed esterno.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gaps {
    pub inner: u32,
    pub outer: u32,
}

impl Default for Gaps {
    fn default() -> Self {
        Self { inner: 8, outer: 8 }
    }
}

/// Modalità di split per Dwindle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitMode {
    #[default]
    LongestSide,
    Alternating,
}

/// Tipi di layout supportati.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutKind {
    #[default]
    Dwindle,
    MasterStack,
}

impl Rect {
    /// Applica il gap esterno una volta sull'area di lavoro.
    pub fn apply_outer_gap(&self, outer: u32) -> Self {
        let double_outer = outer.saturating_mul(2);
        let w = self.width.saturating_sub(double_outer).max(1);
        let h = self.height.saturating_sub(double_outer).max(1);
        let x = self.x.saturating_add(outer as i32);
        let y = self.y.saturating_add(outer as i32);
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    /// Divide in due orizzontalmente (left, right) applicando inner gap.
    pub fn split_horizontal(&self, ratio: f64, inner: u32) -> (Self, Self) {
        let usable_w = self.width.saturating_sub(inner);
        let first_w = ((usable_w as f64) * ratio).floor() as u32;
        let second_w = usable_w.saturating_sub(first_w);

        let first = Rect {
            x: self.x,
            y: self.y,
            width: first_w,
            height: self.height,
        };
        let second = Rect {
            x: self
                .x
                .saturating_add((first_w.saturating_add(inner)) as i32),
            y: self.y,
            width: second_w,
            height: self.height,
        };
        (first, second)
    }

    /// Divide in due verticalmente (top, bottom) applicando inner gap.
    pub fn split_vertical(&self, ratio: f64, inner: u32) -> (Self, Self) {
        let usable_h = self.height.saturating_sub(inner);
        let first_h = ((usable_h as f64) * ratio).floor() as u32;
        let second_h = usable_h.saturating_sub(first_h);

        let first = Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: first_h,
        };
        let second = Rect {
            x: self.x,
            y: self
                .y
                .saturating_add((first_h.saturating_add(inner)) as i32),
            width: self.width,
            height: second_h,
        };
        (first, second)
    }
}

/// Nodo dell'albero binario persistente Dwindle.
#[derive(Debug, Clone)]
enum DwindleNode {
    Leaf,
    Split {
        ratio: f64,
        first: Box<DwindleNode>,
        second: Box<DwindleNode>,
    },
}

impl DwindleNode {
    /// Inserimento nella foglia target MRU (ovvero il ramo second più a destra/in basso).
    fn insert_mru(&mut self, mode: SplitMode, ratio: f64, depth: usize) {
        match self {
            DwindleNode::Leaf => {
                let _ = (mode, depth);
                *self = DwindleNode::Split {
                    ratio,
                    first: Box::new(DwindleNode::Leaf),
                    second: Box::new(DwindleNode::Leaf),
                };
            }
            DwindleNode::Split { second, .. } => {
                second.insert_mru(mode, ratio, depth + 1);
            }
        }
    }

    /// Calcola la lista finale di Rect delle foglie (in ordine DFS).
    fn compute_rects(&self, area: Rect, gaps: &Gaps, mode: SplitMode) -> Vec<Rect> {
        match self {
            DwindleNode::Leaf => vec![area],
            DwindleNode::Split {
                ratio: split_ratio,
                first,
                second,
            } => {
                let is_horiz = match mode {
                    SplitMode::LongestSide => area.width >= area.height,
                    SplitMode::Alternating => area.width >= area.height,
                };

                let (r1, r2) = if is_horiz {
                    area.split_horizontal(*split_ratio, gaps.inner)
                } else {
                    area.split_vertical(*split_ratio, gaps.inner)
                };

                let mut res = first.compute_rects(r1, gaps, mode);
                res.extend(second.compute_rects(r2, gaps, mode));
                res
            }
        }
    }
}

/// Calcola il layout Dwindle per n finestre su una data area.
pub fn compute_dwindle(
    area: Rect,
    gaps: &Gaps,
    count: usize,
    mode: SplitMode,
    ratio: f64,
) -> Vec<Rect> {
    if count == 0 {
        return vec![];
    }
    let usable = area.apply_outer_gap(gaps.outer);
    if count == 1 {
        return vec![usable];
    }

    let mut root = DwindleNode::Leaf;
    for _ in 1..count {
        root.insert_mru(mode, ratio, 0);
    }

    root.compute_rects(usable, gaps, mode)
}

/// Calcola il layout Master-Stack per n finestre su una data area.
pub fn compute_master_stack(area: Rect, gaps: &Gaps, count: usize, mfact: f64) -> Vec<Rect> {
    if count == 0 {
        return vec![];
    }
    let usable = area.apply_outer_gap(gaps.outer);

    let usable_w = usable.width.saturating_sub(gaps.inner);
    let master_w = ((usable_w as f64) * mfact).floor() as u32;

    let master_rect = Rect {
        x: usable.x,
        y: usable.y,
        width: master_w,
        height: usable.height,
    };

    if count == 1 {
        return vec![master_rect];
    }

    let slave_w = usable_w.saturating_sub(master_w);
    let slave_count = count - 1;
    let slave_x = usable
        .x
        .saturating_add((master_w.saturating_add(gaps.inner)) as i32);
    let total_inner = gaps
        .inner
        .saturating_mul((slave_count.saturating_sub(1)) as u32);
    let usable_slave_h = usable.height.saturating_sub(total_inner);

    let base_h = usable_slave_h / (slave_count as u32);
    let rem = usable_slave_h % (slave_count as u32);

    let mut rects = vec![master_rect];
    let mut current_y = usable.y;

    for i in 0..slave_count {
        let h_i = base_h + if (i as u32) < rem { 1 } else { 0 };
        rects.push(Rect {
            x: slave_x,
            y: current_y,
            width: slave_w,
            height: h_i,
        });
        current_y = current_y.saturating_add((h_i.saturating_add(gaps.inner)) as i32);
    }

    rects
}
