// Test minimali della policy di eleggibilità (doc 04 §3).

use ktile::state::eligibility::{
    is_eligible, ExcludeConfig, WindowMeta, WindowStateAtom, WindowType,
};
use ktile::state::registry::WindowId;

fn base_meta() -> WindowMeta {
    WindowMeta {
        id: WindowId(100),
        is_mapped: true,
        window_type: Some(WindowType::Normal),
        transient_for: None,
        states: vec![],
        desktop: 0,
        class: Some("Firefox".to_string()),
        instance: Some("navigator".to_string()),
        title: Some("Browser".to_string()),
    }
}

#[test]
fn test_normal_window_is_eligible() {
    let meta = base_meta();
    let config = ExcludeConfig::default();
    assert!(is_eligible(&meta, 0, &config));
}

#[test]
fn test_unmapped_window_is_not_eligible() {
    let mut meta = base_meta();
    meta.is_mapped = false;
    let config = ExcludeConfig::default();
    assert!(!is_eligible(&meta, 0, &config));
}

#[test]
fn test_dialog_window_is_not_eligible() {
    let mut meta = base_meta();
    meta.window_type = Some(WindowType::Dialog);
    let config = ExcludeConfig::default();
    assert!(!is_eligible(&meta, 0, &config));
}

#[test]
fn test_transient_window_is_not_eligible() {
    let mut meta = base_meta();
    meta.transient_for = Some(WindowId(99));
    let config = ExcludeConfig::default();
    assert!(!is_eligible(&meta, 0, &config));
}

#[test]
fn test_fullscreen_window_is_not_eligible() {
    let mut meta = base_meta();
    meta.states.push(WindowStateAtom::Fullscreen);
    let config = ExcludeConfig::default();
    assert!(!is_eligible(&meta, 0, &config));
}

#[test]
fn test_hidden_window_is_not_eligible() {
    let mut meta = base_meta();
    meta.states.push(WindowStateAtom::Hidden);
    let config = ExcludeConfig::default();
    assert!(!is_eligible(&meta, 0, &config));
}

#[test]
fn test_sticky_window_is_not_eligible_d9() {
    let mut meta = base_meta();
    meta.desktop = 0xFFFFFFFF;
    let config = ExcludeConfig::default();
    assert!(!is_eligible(&meta, 0, &config));
}

#[test]
fn test_other_desktop_window_is_not_eligible() {
    let meta = base_meta(); // desktop = 0
    let config = ExcludeConfig::default();
    assert!(!is_eligible(&meta, 1, &config));
}

#[test]
fn test_excluded_class_is_not_eligible() {
    let meta = base_meta(); // class = "Firefox"
    let config = ExcludeConfig {
        classes: vec!["firefox".to_string()],
        instances: vec![],
    };
    assert!(!is_eligible(&meta, 0, &config));
}
