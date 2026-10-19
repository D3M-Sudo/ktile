// Test del motore geometrico con caricamento delle 25 fixture da docs/milestones/fixtures-geometry.json.

use ktile::layout::geometry::{compute_dwindle, compute_master_stack, Gaps, Rect, SplitMode};
use ktile::layout::plan::Plan;
use ktile::state::registry::WindowId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FixtureRect {
    x: i32,
    y: i32,
    #[serde(alias = "w", alias = "width")]
    width: u32,
    #[serde(alias = "h", alias = "height")]
    height: u32,
}

#[derive(Debug, Deserialize)]
struct FixtureGaps {
    inner: u32,
    outer: u32,
}

#[derive(Debug, Deserialize)]
struct FixturesFile {
    fixtures: Vec<Fixture>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    name: String,
    layout: String,
    area: FixtureRect,
    gaps: FixtureGaps,
    mfact: Option<f64>,
    expected: Option<Vec<FixtureRect>>,
    invariants: Option<bool>,
}

#[test]
fn test_all_25_geometry_fixtures() {
    let json_str = include_str!("../docs/milestones/fixtures-geometry.json");
    let file: FixturesFile =
        serde_json::from_str(json_str).expect("Impossibile parsare fixtures-geometry.json");

    assert_eq!(
        file.fixtures.len(),
        25,
        "Ci devono essere esattamente 25 fixture"
    );

    for f in &file.fixtures {
        let expected_rects: Vec<Rect> = f
            .expected
            .as_ref()
            .expect("Expected rects mancanti")
            .iter()
            .map(|r| Rect {
                x: r.x,
                y: r.y,
                width: r.width,
                height: r.height,
            })
            .collect();

        let count = expected_rects.len();
        let area = Rect {
            x: f.area.x,
            y: f.area.y,
            width: f.area.width,
            height: f.area.height,
        };
        let gaps = Gaps {
            inner: f.gaps.inner,
            outer: f.gaps.outer,
        };

        let calculated = match f.layout.as_str() {
            "dwindle" => {
                let mode = if f.name.contains("alternating") {
                    SplitMode::Alternating
                } else {
                    SplitMode::LongestSide
                };
                compute_dwindle(area, &gaps, count, mode, 0.5)
            }
            "master-stack" => {
                let mfact =
                    f.mfact
                        .unwrap_or_else(|| if f.name.contains("035") { 0.35 } else { 0.55 });
                compute_master_stack(area, &gaps, count, mfact)
            }
            other => panic!("Layout sconosciuto nella fixture {}: {}", f.name, other),
        };

        // Verifica congruenza esatta degli output
        assert_eq!(
            calculated, expected_rects,
            "Fixture '{}' fallita!\nCalcolato: {:?}\nAtteso:    {:?}",
            f.name, calculated, expected_rects
        );

        // Verifica invarianti se richiesto
        if f.invariants.unwrap_or(false) {
            for r in &calculated {
                assert!(
                    r.width >= 1 && r.height >= 1,
                    "Fixture '{}': rettangolo {:?} viola la dimensione minima >= 1x1",
                    f.name,
                    r
                );
            }
        }
    }

    println!("[OK] Tutte le 25 fixture geometriche superate con successo!");
}

#[test]
fn test_plan_idempotence() {
    let rect1 = Rect {
        x: 8,
        y: 8,
        width: 948,
        height: 1024,
    };
    let rect2 = Rect {
        x: 964,
        y: 8,
        width: 948,
        height: 1024,
    };

    let current = vec![(WindowId(1), rect1), (WindowId(2), rect2)];
    let expected = vec![(WindowId(1), rect1), (WindowId(2), rect2)];

    let plan = Plan::diff(&current, &expected);
    assert!(
        plan.is_empty(),
        "Il piano dev'essere vuoto quando la configurazione è identica (idempotenza)"
    );
}
