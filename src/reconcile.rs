//! Pure reconciliation between observed window geometry and a desired layout.
//!
//! This module does not perform X11 requests. A backend must revalidate each
//! action immediately before applying it and call `record_applied` only after
//! the server confirms the operation.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    layout::{LayoutEngine, LayoutError, LayoutKind, Rect},
    model::{WindowId, WindowInfo},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedWindow {
    pub info: WindowInfo,
    pub geometry: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryAction {
    Apply { id: WindowId, from: Rect, to: Rect },
    Restore { id: WindowId, from: Rect, to: Rect },
}

impl GeometryAction {
    pub fn id(&self) -> WindowId {
        match self {
            Self::Apply { id, .. } | Self::Restore { id, .. } => *id,
        }
    }

    pub fn from(&self) -> Rect {
        match self {
            Self::Apply { from, .. } | Self::Restore { from, .. } => *from,
        }
    }

    pub fn to(&self) -> Rect {
        match self {
            Self::Apply { to, .. } | Self::Restore { to, .. } => *to,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Reconciler {
    original_geometry: HashMap<WindowId, Rect>,
    last_applied_geometry: HashMap<WindowId, Rect>,
    user_managed: HashSet<WindowId>,
}

impl Reconciler {
    /// Produce desired geometry actions without performing window-system I/O.
    ///
    /// A geometry that differs from the last confirmed K-tile geometry is
    /// treated as a user/window-manager change. That window is left alone until
    /// it disappears from a snapshot and later reappears.
    pub fn plan(
        &mut self,
        engine: &LayoutEngine,
        layout: LayoutKind,
        area: Rect,
        observed: &[ObservedWindow],
    ) -> Result<Vec<GeometryAction>, LayoutError> {
        let mut next = self.clone();
        let present: HashSet<_> = observed.iter().map(|window| window.info.id).collect();
        next.original_geometry.retain(|id, _| present.contains(id));
        next.last_applied_geometry
            .retain(|id, _| present.contains(id));
        next.user_managed.retain(|id| present.contains(id));

        let by_id: HashMap<_, _> = observed
            .iter()
            .map(|window| (window.info.id, window))
            .collect();
        let mut actions = Vec::new();

        for window in observed {
            let id = window.info.id;
            if next.user_managed.contains(&id) {
                continue;
            }

            if let Some(last_applied) = next.last_applied_geometry.get(&id).copied() {
                if window.geometry != last_applied {
                    next.original_geometry.remove(&id);
                    next.last_applied_geometry.remove(&id);
                    next.user_managed.insert(id);
                    continue;
                }

                if !window.info.is_tile_eligible() {
                    if let Some(original) = next.original_geometry.get(&id).copied() {
                        if original != window.geometry {
                            actions.push(GeometryAction::Restore {
                                id,
                                from: window.geometry,
                                to: original,
                            });
                        }
                    }
                }
            }
        }

        let candidates: Vec<WindowInfo> = observed
            .iter()
            .filter(|window| {
                window.info.is_tile_eligible() && !next.user_managed.contains(&window.info.id)
            })
            .map(|window| window.info.clone())
            .collect();

        let layout_plan = engine.plan(layout, area, &candidates)?;
        for planned in layout_plan.windows {
            let Some(window) = by_id.get(&planned.id) else {
                continue;
            };
            if window.geometry != planned.rect {
                next.original_geometry
                    .entry(planned.id)
                    .or_insert(window.geometry);
                actions.push(GeometryAction::Apply {
                    id: planned.id,
                    from: window.geometry,
                    to: planned.rect,
                });
            }
        }

        *self = next;
        Ok(actions)
    }

    /// Confirm an action only after the backend successfully applied it.
    ///
    /// The backend remains responsible for re-reading current geometry and
    /// eligibility immediately before the X11 request.
    pub fn record_applied(&mut self, action: &GeometryAction) {
        match action {
            GeometryAction::Apply { id, to, .. } => {
                self.last_applied_geometry.insert(*id, *to);
            }
            GeometryAction::Restore { id, .. } => {
                self.original_geometry.remove(id);
                self.last_applied_geometry.remove(id);
                self.user_managed.remove(id);
            }
        }
    }

    /// Propose restoration only for windows still at the last confirmed K-tile
    /// geometry. Never restore over a newer user/window-manager geometry.
    pub fn plan_restore(&self, observed: &[ObservedWindow]) -> Vec<GeometryAction> {
        let by_id: HashMap<_, _> = observed
            .iter()
            .map(|window| (window.info.id, window))
            .collect();
        let mut ids: Vec<_> = self.original_geometry.keys().copied().collect();
        ids.sort_by_key(|id| id.0);

        ids.into_iter()
            .filter_map(|id| {
                let original = *self.original_geometry.get(&id)?;
                let last_applied = *self.last_applied_geometry.get(&id)?;
                let window = by_id.get(&id)?;
                if !window.info.is_tile_eligible()
                    || window.geometry != last_applied
                    || window.geometry == original
                {
                    return None;
                }
                Some(GeometryAction::Restore {
                    id,
                    from: window.geometry,
                    to: original,
                })
            })
            .collect()
    }

    #[cfg(test)]
    fn is_user_managed(&self, id: WindowId) -> bool {
        self.user_managed.contains(&id)
    }

    #[cfg(test)]
    fn original_geometry(&self, id: WindowId) -> Option<Rect> {
        self.original_geometry.get(&id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::WindowState;

    fn area() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 1200,
            height: 800,
        }
    }

    fn observed(count: u32) -> Vec<ObservedWindow> {
        (1..=count)
            .map(|id| ObservedWindow {
                info: WindowInfo {
                    id: WindowId(id),
                    title: None,
                    app_class: None,
                    workspace: 0,
                    monitor: 0,
                    state: WindowState::default(),
                    floating: false,
                },
                geometry: Rect {
                    x: 20 * id as i32,
                    y: 30,
                    width: 400,
                    height: 300,
                },
            })
            .collect()
    }

    #[test]
    fn plans_apply_actions_and_captures_original_geometry() {
        let mut reconciler = Reconciler::default();
        let windows = observed(2);
        let actions = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &windows,
            )
            .unwrap();

        assert_eq!(actions.len(), 2);
        assert_eq!(
            reconciler.original_geometry(WindowId(1)),
            Some(windows[0].geometry)
        );
        assert!(
            actions
                .iter()
                .all(|action| matches!(action, GeometryAction::Apply { .. }))
        );
    }

    #[test]
    fn restore_is_planned_only_for_confirmed_geometry() {
        let mut reconciler = Reconciler::default();
        let original = observed(1);
        let actions = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &original,
            )
            .unwrap();
        let apply = actions[0].clone();
        reconciler.record_applied(&apply);

        let tiled = vec![ObservedWindow {
            info: original[0].info.clone(),
            geometry: apply.to(),
        }];
        let restore = reconciler.plan_restore(&tiled);
        assert_eq!(restore.len(), 1);
        assert_eq!(restore[0].from(), apply.to());
        assert_eq!(restore[0].to(), original[0].geometry);
    }

    #[test]
    fn user_geometry_changes_suppress_future_tiling_and_restore() {
        let mut reconciler = Reconciler::default();
        let initial = observed(1);
        let apply = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &initial,
            )
            .unwrap()
            .remove(0);
        reconciler.record_applied(&apply);

        let changed_geometry = Rect {
            x: 900,
            y: 100,
            width: 250,
            height: 220,
        };
        let changed = vec![ObservedWindow {
            info: initial[0].info.clone(),
            geometry: changed_geometry,
        }];
        let actions = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &changed,
            )
            .unwrap();

        assert!(actions.iter().all(|action| action.id() != WindowId(1)));
        assert!(reconciler.is_user_managed(WindowId(1)));
        assert!(reconciler.plan_restore(&changed).is_empty());
    }


    #[test]
    fn settled_geometry_is_idempotent() {
        let mut reconciler = Reconciler::default();
        let original = observed(3);
        let first = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::Grid,
                area(),
                &original,
            )
            .unwrap();
        for action in &first {
            reconciler.record_applied(action);
        }

        let settled: Vec<_> = original
            .iter()
            .map(|window| ObservedWindow {
                info: window.info.clone(),
                geometry: first
                    .iter()
                    .find(|action| action.id() == window.info.id)
                    .map(GeometryAction::to)
                    .unwrap_or(window.geometry),
            })
            .collect();
        let second = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::Grid,
                area(),
                &settled,
            )
            .unwrap();

        assert!(second.is_empty());
    }

    #[test]
    fn ineligible_window_proposes_safe_restore() {
        let mut reconciler = Reconciler::default();
        let original = observed(1);
        let apply = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &original,
            )
            .unwrap()
            .remove(0);
        reconciler.record_applied(&apply);

        let mut info = original[0].info.clone();
        info.state.fullscreen = true;
        let fullscreen = vec![ObservedWindow {
            info,
            geometry: apply.to(),
        }];
        let actions = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &fullscreen,
            )
            .unwrap();

        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], GeometryAction::Restore { .. }));
        assert_eq!(actions[0].to(), original[0].geometry);
    }

    #[test]
    fn disappeared_windows_are_removed_from_tracking() {
        let mut reconciler = Reconciler::default();
        let initial = observed(1);
        let _ = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &initial,
            )
            .unwrap();

        let _ = reconciler
            .plan(
                &LayoutEngine::default(),
                LayoutKind::VerticalLeft,
                area(),
                &[],
            )
            .unwrap();

        assert_eq!(reconciler.original_geometry(WindowId(1)), None);
    }
}
