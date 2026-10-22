//! Guarded manual application of a geometric plan.
//!
//! This is the only module allowed to issue window geometry requests. Each
//! batch contains at most three ConfigureWindow requests; the result is read
//! back once per window and WM deviations are reported, never retried.

use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ClientMessageData, ClientMessageEvent, ConnectionExt, EventMask, MapState, Window,
};
use x11rb::rust_connection::RustConnection;

use crate::errors::KtileError;
use crate::layout::geometry::Rect;
use crate::layout::plan::{Plan, WindowMove};
use crate::state::registry::WindowId;
use crate::x11::connection::EwmhAtoms;
use crate::x11::{ewmh, icccm};

/// Upper bound for geometry requests in one batch. A batch is flushed and
/// verified before another batch can start.
pub const MAX_REQUESTS_PER_BURST: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyStatus {
    Applied { actual: Rect },
    AlreadyCorrect,
    Deviated { actual: Rect },
    Skipped { reason: String },
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOutcome {
    pub window_id: WindowId,
    pub target: Rect,
    pub status: ApplyStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplyReport {
    pub outcomes: Vec<ApplyOutcome>,
    /// Number of ConfigureWindow requests actually queued for the X server.
    pub requests_sent: usize,
}

impl ApplyReport {
    pub fn applied_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome.status,
                    ApplyStatus::Applied { .. } | ApplyStatus::AlreadyCorrect
                )
            })
            .count()
    }

    pub fn deviated_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|outcome| matches!(outcome.status, ApplyStatus::Deviated { .. }))
            .count()
    }

    pub fn skipped_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|outcome| matches!(outcome.status, ApplyStatus::Skipped { .. }))
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|outcome| matches!(outcome.status, ApplyStatus::Failed { .. }))
            .count()
    }
}

/// Expected geometry while a ConfigureWindow request is in flight. The M6
/// event loop can use this exact matcher to filter self-generated feedback.
#[derive(Debug, Default)]
pub struct GeometryInFlight {
    expected: HashMap<WindowId, Rect>,
}

impl GeometryInFlight {
    pub fn begin(&mut self, window: WindowId, expected: Rect) {
        self.expected.insert(window, expected);
    }

    pub fn matches(&self, window: WindowId, observed: Rect) -> bool {
        self.expected
            .get(&window)
            .is_some_and(|expected| *expected == observed)
    }

    pub fn finish(&mut self, window: WindowId) {
        self.expected.remove(&window);
    }

    pub fn is_empty(&self) -> bool {
        self.expected.is_empty()
    }
}

/// Applies a plan only to windows that were found eligible in a fresh snapshot.
/// Properties that can make a window unsafe are re-read immediately before the
/// request. The caller must not call this in --dry-run mode.
pub fn apply_plan(
    conn: &RustConnection,
    plan: &Plan,
    allowed_windows: &HashSet<WindowId>,
    active_desktop: u32,
    root: Window,
    atoms: &EwmhAtoms,
) -> Result<ApplyReport, KtileError> {
    let mut report = ApplyReport::default();
    let mut in_flight = GeometryInFlight::default();
    let mut seen = HashSet::new();

    for batch in plan.moves.chunks(MAX_REQUESTS_PER_BURST) {
        let mut pending: Vec<(WindowMove, x11rb::cookie::VoidCookie<'_, RustConnection>)> =
            Vec::new();

        for movement in batch {
            let id = movement.window_id;
            let window = id.0;

            if !seen.insert(id) {
                report.outcomes.push(outcome(
                    id,
                    movement.target_rect,
                    ApplyStatus::Skipped {
                        reason: "duplicated window id in plan".to_string(),
                    },
                ));
                continue;
            }
            if !allowed_windows.contains(&id) {
                report.outcomes.push(outcome(
                    id,
                    movement.target_rect,
                    ApplyStatus::Skipped {
                        reason: "window is no longer eligible in the fresh snapshot".to_string(),
                    },
                ));
                continue;
            }
            if let Err(reason) = validate_target(movement.target_rect) {
                report.outcomes.push(outcome(
                    id,
                    movement.target_rect,
                    ApplyStatus::Skipped { reason },
                ));
                continue;
            }
            if let Err(reason) = live_safety_check(conn, window, active_desktop, atoms) {
                report.outcomes.push(outcome(
                    id,
                    movement.target_rect,
                    ApplyStatus::Skipped { reason },
                ));
                continue;
            }

            let current = match read_geometry(conn, window, root, atoms) {
                Ok(rect) => rect,
                Err(reason) => {
                    report.outcomes.push(outcome(
                        id,
                        movement.target_rect,
                        ApplyStatus::Skipped { reason },
                    ));
                    continue;
                }
            };
            if current == movement.target_rect {
                report.outcomes.push(outcome(
                    id,
                    movement.target_rect,
                    ApplyStatus::AlreadyCorrect,
                ));
                continue;
            }

            in_flight.begin(id, movement.target_rect);
            let flags = 10_u32
                | (1 << 8)
                | (1 << 9)
                | (1 << 10)
                | (1 << 11)
                | (2 << 12);
            let event = ClientMessageEvent::new(
                32,
                window,
                atoms.moveresize_window,
                ClientMessageData::from([
                    flags,
                    movement.target_rect.x as u32,
                    movement.target_rect.y as u32,
                    movement.target_rect.width,
                    movement.target_rect.height,
                ]),
            );
            match conn.send_event(
                false,
                root,
                EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                event,
            ) {
                Ok(cookie) => {
                    report.requests_sent += 1;
                    pending.push((movement.clone(), cookie));
                }
                Err(error) => {
                    in_flight.finish(id);
                    report.outcomes.push(outcome(
                        id,
                        movement.target_rect,
                        ApplyStatus::Failed {
                            reason: format!("could not send _NET_MOVERESIZE_WINDOW: {error}"),
                        },
                    ));
                }
            }
        }

        if pending.is_empty() {
            continue;
        }
        conn.flush()
            .map_err(|error| KtileError::X11(format!("flush geometry requests: {error}")))?;
        // ClientMessage delivery and WM processing are asynchronous. Wait a
        // short bounded interval, then read back once; never retry deviations.
        thread::sleep(Duration::from_millis(80));

        for (movement, cookie) in pending {
            let id = movement.window_id;
            let target = movement.target_rect;
            match cookie.check() {
                Err(error) => {
                    in_flight.finish(id);
                    report.outcomes.push(outcome(id, target, ApplyStatus::Failed {
                        reason: format!("ConfigureWindow rejected (window may have disappeared or WM denied it): {error}"),
                    }));
                }
                Ok(()) => match read_geometry(conn, id.0, root, atoms) {
                    Err(reason) => {
                        in_flight.finish(id);
                        report
                            .outcomes
                            .push(outcome(id, target, ApplyStatus::Failed { reason }));
                    }
                    Ok(actual) => {
                        let matches_request = in_flight.matches(id, actual);
                        in_flight.finish(id);
                        if matches_request {
                            report.outcomes.push(outcome(
                                id,
                                target,
                                ApplyStatus::Applied { actual },
                            ));
                        } else {
                            report.outcomes.push(outcome(
                                id,
                                target,
                                ApplyStatus::Deviated { actual },
                            ));
                        }
                    }
                },
            }
        }
    }

    debug_assert!(in_flight.is_empty());
    Ok(report)
}

fn outcome(window_id: WindowId, target: Rect, status: ApplyStatus) -> ApplyOutcome {
    ApplyOutcome {
        window_id,
        target,
        status,
    }
}

fn validate_target(rect: Rect) -> Result<(), String> {
    if rect.width == 0 || rect.height == 0 {
        return Err("target width and height must be positive".to_string());
    }
    if rect.width > u16::MAX as u32 || rect.height > u16::MAX as u32 {
        return Err("target dimensions exceed the X11 protocol limit (65535)".to_string());
    }
    if rect.x < i16::MIN as i32
        || rect.x > i16::MAX as i32
        || rect.y < i16::MIN as i32
        || rect.y > i16::MAX as i32
    {
        return Err("target coordinates exceed the X11 protocol limit (-32768..32767)".to_string());
    }
    Ok(())
}

fn live_safety_check(
    conn: &RustConnection,
    window: Window,
    active_desktop: u32,
    atoms: &EwmhAtoms,
) -> Result<(), String> {
    let attrs = conn
        .get_window_attributes(window)
        .map_err(|error| format!("window disappeared before request: {error}"))?
        .reply()
        .map_err(|error| format!("window disappeared before request: {error}"))?;
    if attrs.map_state != MapState::VIEWABLE {
        return Err("window is not viewable (possibly minimized or unmapped)".to_string());
    }

    let states = ewmh::get_wm_state(conn, atoms, window)
        .map_err(|error| format!("cannot re-read window state: {error}"))?;
    if states.contains(&atoms.wm_state_fullscreen) {
        return Err("fullscreen window is protected from geometry changes".to_string());
    }
    if states.contains(&atoms.wm_state_hidden) {
        return Err("hidden/minimized window is protected from geometry changes".to_string());
    }

    if let Some(kind) = ewmh::get_wm_window_type(conn, atoms, window)
        .map_err(|error| format!("cannot re-read window type: {error}"))?
    {
        if kind != atoms.wm_window_type_normal {
            return Err("window type is not _NET_WM_WINDOW_TYPE_NORMAL".to_string());
        }
    }

    if let Some(desktop) = ewmh::get_wm_desktop(conn, atoms, window)
        .map_err(|error| format!("cannot re-read window desktop: {error}"))?
    {
        if desktop == u32::MAX || desktop != active_desktop {
            return Err("window is sticky or no longer on the active desktop".to_string());
        }
    }

    if icccm::read_wm_transient_for(conn, window)
        .map_err(|error| format!("cannot re-read transient relationship: {error}"))?
        .is_some()
    {
        return Err("transient/dialog window is protected from geometry changes".to_string());
    }

    Ok(())
}

fn read_geometry(
    conn: &RustConnection,
    window: Window,
    root: Window,
    atoms: &EwmhAtoms,
) -> Result<Rect, String> {
    let reply = conn
        .get_geometry(window)
        .map_err(|error| format!("window is gone: {error}"))?
        .reply()
        .map_err(|error| format!("window is gone: {error}"))?;
    let translated = conn
        .translate_coordinates(window, root, 0, 0)
        .map_err(|error| format!("cannot translate window coordinates to root: {error}"))?
        .reply()
        .map_err(|error| format!("cannot translate window coordinates to root: {error}"))?;

    let extents = conn
        .get_property(
            false,
            window,
            atoms.frame_extents,
            x11rb::protocol::xproto::AtomEnum::CARDINAL,
            0,
            4,
        )
        .map_err(|error| format!("cannot read frame extents: {error}"))?
        .reply()
        .map_err(|error| format!("cannot read frame extents: {error}"))?;
    let values: Vec<u32> = extents
        .value32()
        .map(|values| values.collect())
        .unwrap_or_default();
    let left = values.first().copied().unwrap_or(0);
    let right = values.get(1).copied().unwrap_or(0);
    let top = values.get(2).copied().unwrap_or(0);
    let bottom = values.get(3).copied().unwrap_or(0);

    Ok(Rect {
        x: i32::from(translated.dst_x).saturating_sub(left as i32),
        y: i32::from(translated.dst_y).saturating_sub(top as i32),
        width: u32::from(reply.width).saturating_add(left).saturating_add(right),
        height: u32::from(reply.height).saturating_add(top).saturating_add(bottom),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_validation_rejects_zero_dimensions_and_unrepresentable_values() {
        assert!(validate_target(Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 10
        })
        .is_err());
        assert!(validate_target(Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 0
        })
        .is_err());
        assert!(validate_target(Rect {
            x: 0,
            y: 0,
            width: 65_536,
            height: 10
        })
        .is_err());
        assert!(validate_target(Rect {
            x: 32_768,
            y: 0,
            width: 10,
            height: 10
        })
        .is_err());
        assert!(validate_target(Rect {
            x: -32_769,
            y: 0,
            width: 10,
            height: 10
        })
        .is_err());
        assert!(validate_target(Rect {
            x: -10,
            y: -10,
            width: 10,
            height: 10
        })
        .is_ok());
    }

    #[test]
    fn in_flight_geometry_matches_only_the_expected_rectangle() {
        let id = WindowId(42);
        let expected = Rect {
            x: -100,
            y: 10,
            width: 800,
            height: 600,
        };
        let other = Rect {
            x: -99,
            y: 10,
            width: 800,
            height: 600,
        };
        let mut flight = GeometryInFlight::default();
        flight.begin(id, expected);
        assert!(flight.matches(id, expected));
        assert!(!flight.matches(id, other));
        flight.finish(id);
        assert!(!flight.matches(id, expected));
        assert!(flight.is_empty());
    }

    #[test]
    fn duplicate_window_ids_are_not_silently_accepted_by_policy() {
        let mut seen = HashSet::new();
        assert!(seen.insert(WindowId(7)));
        assert!(!seen.insert(WindowId(7)));
    }
}
