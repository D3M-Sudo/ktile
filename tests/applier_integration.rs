//! Manual geometry applier integration tests against an isolated X server.
//!
//! Run with: cargo test --test applier_integration -- --ignored --nocapture --test-threads=1
//! These tests do not certify Muffin behavior; local Cinnamon/Muffin checks are
//! separately tracked in docs/ROADMAP.md.

use std::collections::HashSet;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use ktile::layout::geometry::Rect;
use ktile::layout::plan::{Plan, WindowMove};
use ktile::state::registry::WindowId;
use ktile::x11::applier::{apply_plan, ApplyStatus};
use ktile::x11::connection::{connect, EwmhAtoms};
use ktile::x11::ewmh::verify_wm;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt, CreateWindowAux, EventMask, PropMode, WindowClass,
};

struct XvfbGuard(Child);

impl Drop for XvfbGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "requires Xvfb; executed explicitly by CI"]
fn applies_live_window_and_safely_skips_window_destroyed_before_request() {
    let child = Command::new("Xvfb")
        .args([":96", "-screen", "0", "1280x800x24", "-nolisten", "tcp"])
        .spawn()
        .expect("Xvfb must be installed in the CI test runner");
    let _xvfb_guard = XvfbGuard(child);

    let old_display = std::env::var("DISPLAY").ok();
    std::env::set_var("DISPLAY", ":96");

    let wm = Command::new("openbox")
        .env("DISPLAY", ":96")
        .spawn()
        .expect("Openbox must be installed in the CI test runner");
    let _wm_guard = XvfbGuard(wm);

    let connection = (0..40)
        .find_map(|_| {
            thread::sleep(Duration::from_millis(100));
            connect().ok()
        })
        .expect("Xvfb :96 should accept connections");
    let (conn, screen_num) = connection;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let atoms = EwmhAtoms::new(&conn).expect("EWMH atoms should intern");
    let wm_ready = (0..40).any(|_| {
        thread::sleep(Duration::from_millis(100));
        verify_wm(&conn, root, &atoms).is_ok()
    });
    assert!(wm_ready, "Openbox should publish EWMH properties");

    let live = conn.generate_id().expect("allocate live XID");
    conn.create_window(
        0,
        live,
        root,
        10,
        20,
        100,
        80,
        0,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &CreateWindowAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
    )
    .expect("create test window")
    .check()
    .expect("create test window should succeed");
    conn.change_property32(
        PropMode::REPLACE,
        live,
        atoms.wm_window_type,
        AtomEnum::ATOM,
        &[atoms.wm_window_type_normal],
    )
    .expect("set normal window type")
    .check()
    .expect("normal window type property should succeed");
    conn.change_property32(
        PropMode::REPLACE,
        live,
        atoms.wm_desktop,
        AtomEnum::CARDINAL,
        &[0],
    )
    .expect("set active desktop")
    .check()
    .expect("desktop property should succeed");
    conn.map_window(live)
        .expect("map test window")
        .check()
        .expect("map test window should succeed");
    conn.flush().expect("flush test setup");
    thread::sleep(Duration::from_millis(500));

    // Allocate an XID but deliberately never create the corresponding window.
    let dead = conn.generate_id().expect("allocate dead XID");
    let plan = Plan::new(vec![
        WindowMove {
            window_id: WindowId(live),
            target_rect: Rect {
                x: 30,
                y: 40,
                width: 200,
                height: 150,
            },
        },
        WindowMove {
            window_id: WindowId(dead),
            target_rect: Rect {
                x: 300,
                y: 200,
                width: 200,
                height: 150,
            },
        },
    ]);
    let allowed = HashSet::from([WindowId(live), WindowId(dead)]);
    let report = apply_plan(&conn, &plan, &allowed, 0, root, &atoms)
        .expect("per-window X errors should be reported without aborting the batch");

    assert_eq!(
        report.requests_sent, 1,
        "only the live window may receive a request"
    );
    assert_eq!(report.outcomes.len(), 2);
    assert!(report.outcomes.iter().any(|o| {
        o.window_id == WindowId(live) && matches!(o.status, ApplyStatus::Applied { .. })
    }));
    assert!(report.outcomes.iter().any(|o| {
        o.window_id == WindowId(dead) && matches!(o.status, ApplyStatus::Skipped { .. })
    }));

    let _ = conn.destroy_window(live);
    let _ = conn.flush();
    if let Some(display) = old_display {
        std::env::set_var("DISPLAY", display);
    } else {
        std::env::remove_var("DISPLAY");
    }
}
