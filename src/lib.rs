//! K-tile's backend-independent domain model and deterministic layout engine.
//!
//! The core deliberately does not talk to X11 or GTK. It can therefore be tested
//! without a running desktop session and reused by a future Wayland backend.

pub mod config;
pub mod layout;
pub mod model;
pub mod reconcile;

#[cfg(feature = "x11")]
pub mod x11;

pub use layout::{LayoutEngine, LayoutError, LayoutKind, LayoutPlan, Rect};
pub use model::{WindowId, WindowInfo, WindowState};
pub use reconcile::{GeometryAction, ObservedWindow, Reconciler};
