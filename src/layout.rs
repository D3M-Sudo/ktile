use crate::model::{WindowId, WindowInfo};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn right(self) -> i64 {
        i64::from(self.x) + i64::from(self.width)
    }

    pub fn bottom(self) -> i64 {
        i64::from(self.y) + i64::from(self.height)
    }

    pub fn is_valid(self) -> bool {
        self.width > 0 && self.height > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutKind {
    #[default]
    VerticalLeft,
    VerticalRight,
    HorizontalTop,
    HorizontalBottom,
    Maximized,
    Grid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedWindow {
    pub id: WindowId,
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutPlan {
    pub area: Rect,
    pub layout: LayoutKind,
    pub windows: Vec<PlannedWindow>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LayoutError {
    #[error("work area must have non-zero width and height")]
    InvalidWorkArea,
    #[error("master ratio must be between 0.1 and 0.9")]
    InvalidMasterRatio,
    #[error("work area is too small for the requested layout and window count")]
    InsufficientWorkArea,
}

#[derive(Debug, Clone)]
pub struct LayoutEngine {
    pub master_ratio: f64,
    pub master_count: usize,
    pub gap: u32,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            master_ratio: 0.58,
            master_count: 1,
            gap: 8,
        }
    }
}

impl LayoutEngine {
    pub fn plan(
        &self,
        layout: LayoutKind,
        area: Rect,
        windows: &[WindowInfo],
    ) -> Result<LayoutPlan, LayoutError> {
        if !area.is_valid() {
            return Err(LayoutError::InvalidWorkArea);
        }
        if !(0.1..=0.9).contains(&self.master_ratio) {
            return Err(LayoutError::InvalidMasterRatio);
        }

        let eligible: Vec<_> = windows.iter().filter(|w| w.is_tile_eligible()).collect();
        let rects = match layout {
            LayoutKind::VerticalLeft | LayoutKind::VerticalRight => {
                self.master_stack(&eligible, area, true, layout == LayoutKind::VerticalRight)
            }
            LayoutKind::HorizontalTop | LayoutKind::HorizontalBottom => self.master_stack(
                &eligible,
                area,
                false,
                layout == LayoutKind::HorizontalBottom,
            ),
            LayoutKind::Maximized => eligible.iter().map(|_| area).collect(),
            LayoutKind::Grid => self.grid(&eligible, area),
        };

        if rects.iter().any(|rect| {
            !rect.is_valid()
                || rect.x < area.x
                || rect.y < area.y
                || rect.right() > area.right()
                || rect.bottom() > area.bottom()
        }) {
            return Err(LayoutError::InsufficientWorkArea);
        }

        Ok(LayoutPlan {
            area,
            layout,
            windows: eligible
                .iter()
                .zip(rects)
                .map(|(window, rect)| PlannedWindow {
                    id: window.id,
                    rect,
                })
                .collect(),
        })
    }

    fn master_stack(
        &self,
        windows: &[&WindowInfo],
        area: Rect,
        vertical: bool,
        master_last: bool,
    ) -> Vec<Rect> {
        if windows.is_empty() {
            return Vec::new();
        }
        if windows.len() == 1 {
            return vec![area];
        }

        let masters = self.master_count.max(1).min(windows.len() - 1);
        let axis_total = if vertical { area.width } else { area.height };
        let cross_total = if vertical { area.height } else { area.width };
        let axis_gap = self.gap.min(axis_total.saturating_sub(2));
        let available_axis = axis_total.saturating_sub(axis_gap);
        let master_axis = ((available_axis as f64 * self.master_ratio).round() as u32)
            .clamp(1, available_axis.saturating_sub(1).max(1));
        let stack_axis = available_axis.saturating_sub(master_axis);
        let axis_origin = if vertical { area.x } else { area.y };
        let master_origin = if master_last {
            axis_origin.saturating_add(stack_axis.saturating_add(axis_gap) as i32)
        } else {
            axis_origin
        };
        let stack_origin = if master_last {
            axis_origin
        } else {
            axis_origin.saturating_add(master_axis.saturating_add(axis_gap) as i32)
        };

        let mut result = Vec::with_capacity(windows.len());
        for index in 0..masters {
            let start = index as u32 * cross_total / masters as u32;
            let end = (index + 1) as u32 * cross_total / masters as u32;
            let cross_gap = self.gap.min(end.saturating_sub(start).saturating_sub(1));
            let cross_size = end.saturating_sub(start).saturating_sub(cross_gap);
            result.push(if vertical {
                Rect {
                    x: master_origin,
                    y: area.y.saturating_add(start as i32),
                    width: master_axis,
                    height: cross_size,
                }
            } else {
                Rect {
                    x: area.x.saturating_add(start as i32),
                    y: master_origin,
                    width: cross_size,
                    height: master_axis,
                }
            });
        }

        let stack_count = windows.len() - masters;
        for index in 0..stack_count {
            let start = index as u32 * cross_total / stack_count as u32;
            let end = (index + 1) as u32 * cross_total / stack_count as u32;
            let cross_gap = self.gap.min(end.saturating_sub(start).saturating_sub(1));
            let cross_size = end.saturating_sub(start).saturating_sub(cross_gap);
            result.push(if vertical {
                Rect {
                    x: stack_origin,
                    y: area.y.saturating_add(start as i32),
                    width: stack_axis,
                    height: cross_size,
                }
            } else {
                Rect {
                    x: area.x.saturating_add(start as i32),
                    y: stack_origin,
                    width: cross_size,
                    height: stack_axis,
                }
            });
        }
        result
    }

    fn grid(&self, windows: &[&WindowInfo], area: Rect) -> Vec<Rect> {
        if windows.is_empty() {
            return Vec::new();
        }
        let cols = (windows.len() as f64).sqrt().ceil() as u32;
        let rows = (windows.len() as u32).div_ceil(cols);
        windows
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let col = i as u32 % cols;
                let row = i as u32 / cols;
                let x0 = col * area.width / cols;
                let x1 = (col + 1) * area.width / cols;
                let y0 = row * area.height / rows;
                let y1 = (row + 1) * area.height / rows;
                let cell_width = x1.saturating_sub(x0);
                let cell_height = y1.saturating_sub(y0);
                Rect {
                    x: area.x.saturating_add(x0 as i32),
                    y: area.y.saturating_add(y0 as i32),
                    width: cell_width.saturating_sub(self.gap.min(cell_width.saturating_sub(1))),
                    height: cell_height
                        .saturating_sub(self.gap.min(cell_height.saturating_sub(1))),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{WindowInfo, WindowState};

    fn windows(count: u32) -> Vec<WindowInfo> {
        (1..=count)
            .map(|id| WindowInfo {
                id: WindowId(id),
                title: None,
                app_class: None,
                workspace: 0,
                monitor: 0,
                state: WindowState::default(),
                floating: false,
            })
            .collect()
    }

    fn area() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    #[test]
    fn empty_window_set_is_valid() {
        let plan = LayoutEngine::default()
            .plan(LayoutKind::VerticalLeft, area(), &[])
            .unwrap();
        assert!(plan.windows.is_empty());
    }

    #[test]
    fn single_window_uses_whole_area() {
        let work_area = Rect {
            x: 10,
            y: 20,
            width: 1200,
            height: 800,
        };
        let plan = LayoutEngine::default()
            .plan(LayoutKind::VerticalLeft, work_area, &windows(1))
            .unwrap();
        assert_eq!(plan.windows[0].rect, work_area);
    }

    #[test]
    fn floating_and_special_windows_are_never_planned() {
        let mut list = windows(3);
        list[1].floating = true;
        list[2].state.fullscreen = true;
        let plan = LayoutEngine::default()
            .plan(LayoutKind::Grid, area(), &list)
            .unwrap();
        assert_eq!(plan.windows.len(), 1);
        assert_eq!(plan.windows[0].id, WindowId(1));
    }

    #[test]
    fn all_layouts_keep_positive_dimensions() {
        for kind in [
            LayoutKind::VerticalLeft,
            LayoutKind::VerticalRight,
            LayoutKind::HorizontalTop,
            LayoutKind::HorizontalBottom,
            LayoutKind::Maximized,
            LayoutKind::Grid,
        ] {
            let plan = LayoutEngine::default()
                .plan(kind, area(), &windows(7))
                .unwrap();
            assert_eq!(plan.windows.len(), 7);
            assert!(plan.windows.iter().all(|w| w.rect.is_valid()));
        }
    }

    #[test]
    fn master_last_layout_stays_inside_work_area() {
        let work_area = Rect {
            x: 20,
            y: 30,
            width: 1200,
            height: 800,
        };
        for kind in [LayoutKind::VerticalRight, LayoutKind::HorizontalBottom] {
            let plan = LayoutEngine::default()
                .plan(kind, work_area, &windows(5))
                .unwrap();
            assert!(plan.windows.iter().all(|w| {
                w.rect.x >= work_area.x
                    && w.rect.y >= work_area.y
                    && w.rect.right() <= work_area.right()
                    && w.rect.bottom() <= work_area.bottom()
            }));
        }
    }

    #[test]
    fn rejects_invalid_area_and_ratio() {
        assert_eq!(
            LayoutEngine::default().plan(
                LayoutKind::Grid,
                Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 10,
                },
                &[]
            ),
            Err(LayoutError::InvalidWorkArea)
        );
        let engine = LayoutEngine {
            master_ratio: 0.0,
            ..Default::default()
        };
        assert_eq!(
            engine.plan(
                LayoutKind::Grid,
                Rect {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                &[]
            ),
            Err(LayoutError::InvalidMasterRatio)
        );
    }
}
