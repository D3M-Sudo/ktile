// Test M3: layout Master-Stack — master/stack, promozione, mfact clamp, D8. Senza X11.

use ktile::layout::tree::{Dwindle, DwindleConfig, InsertOutcome, ResumeOutcome, SplitDir, SplitMode, SplitTarget, Tree};
use ktile::layout::geometry::{compute_master_stack, Gaps, Rect};
use ktile::layout::master_stack::{
    MasterStack, MasterStackConfig, NewWindowPlacement,
};
use ktile::state::registry::WindowId;

fn w(n: u32) -> WindowId {
    WindowId(n)
}

fn area() -> Rect {
    Rect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    }
}

fn gaps() -> Gaps {
    Gaps { inner: 8, outer: 8 }
}

fn ms_with(n: u32) -> MasterStack {
    let mut ms = MasterStack::new(MasterStackConfig::default());
    for i in 1..=n {
        ms.insert(w(i));
    }
    ms
}

#[test]
fn prima_finestra_diventa_master() {
    let mut ms = MasterStack::new(MasterStackConfig::default());
    assert!(ms.insert(w(1)));
    assert_eq!(ms.master(), Some(w(1)));
    assert!(ms.stack().is_empty());
    assert_eq!(ms.windows_dfs(), vec![w(1)]);
}

#[test]
fn new_master_spinge_il_vecchio_in_stack() {
    let mut ms = MasterStack::new(MasterStackConfig::default());
    ms.insert(w(1));
    ms.insert(w(2));
    // Default: la finestra nuova è il master, il vecchio master diventa testa dello stack.
    assert_eq!(ms.master(), Some(w(2)));
    assert_eq!(ms.stack(), &[w(1)]);
    ms.insert(w(3));
    assert_eq!(ms.master(), Some(w(3)));
    assert_eq!(ms.stack(), &[w(2), w(1)]);
}


#[test]
fn chiusura_master_promuove_il_primo_dello_stack() {
    let mut ms = ms_with(3);
    assert_eq!(ms.master(), Some(w(3)));
    assert_eq!(ms.stack(), &[w(2), w(1)]);
    assert!(ms.close(w(3)));
    // Promozione: il primo dello stack diventa master, nessun riequilibrio.
    assert_eq!(ms.master(), Some(w(2)));
    assert_eq!(ms.stack(), &[w(1)]);
}

#[test]
fn chiusura_finestra_stack_la_rimuove() {
    let mut ms = ms_with(3);
    assert!(ms.close(w(2)));
    assert_eq!(ms.master(), Some(w(3)));
    assert_eq!(ms.stack(), &[w(1)]);
    // Chiusura di una finestra assente: false.
    assert!(!ms.close(w(99)));
}

#[test]
fn chiusura_ultima_finestra_svuota() {
    let mut ms = ms_with(1);
    assert!(ms.close(w(1)));
    assert_eq!(ms.master(), None);
    assert!(ms.stack().is_empty());
    assert!(ms.windows_dfs().is_empty());
}

#[test]
fn mfact_clamp_250_750() {
    // Fuori range → clamp, nessun panic.
    let mut cfg = MasterStackConfig::default();
    cfg.mfact = 0.90;
    assert_eq!(cfg.mfact_milli(), 750);
    cfg.mfact = 0.10;
    assert_eq!(cfg.mfact_milli(), 250);
    cfg.mfact = 0.55;
    assert_eq!(cfg.mfact_milli(), 550);
}

#[test]
fn geometria_conferma_fixture_m2_master_n2() {
    // La pipeline nuova deve riprodurre la fixture M2 `master-n2` (mfact 0.55).
    let ms = ms_with(2);
    let got: Vec<Rect> = ms
        .compute_rects(area(), &gaps())
        .into_iter()
        .map(|(_, r)| r)
        .collect();
    let expected = compute_master_stack(area(), &gaps(), 2, 0.55);
    assert_eq!(got, expected);
}

#[test]
fn geometria_conferma_fixture_m2_master_n3() {
    let ms = ms_with(4);
    let got: Vec<Rect> = ms
        .compute_rects(area(), &gaps())
        .into_iter()
        .map(|(_, r)| r)
        .collect();
    let expected = compute_master_stack(area(), &gaps(), 4, 0.55);
    assert_eq!(got, expected);
}

#[test]
fn d8_sospensione_master_rientro_in_posizione() {
    let mut ms = ms_with(3);
    let prima = ms.windows_dfs();
    assert!(ms.suspend(w(3)));
    // Promozione mentre è sospeso.
    assert_eq!(ms.master(), Some(w(2)));
    // Rientro: slot identificabile (nessuna variazione di lista) → posizione originaria.
    let outcome = ms.resume(w(3), None, None);
    assert_eq!(outcome, ResumeOutcome::InPlace);
    assert_eq!(ms.windows_dfs(), prima, "rientro D8 = stessa sequenza di partenza");
}

#[test]
fn d8_sospensione_stack_rientro_in_posizione() {
    let mut ms = ms_with(3);
    let prima = ms.windows_dfs();
    assert!(ms.suspend(w(2)));
    assert_eq!(ms.windows_dfs(), vec![w(3), w(1)]);
    let outcome = ms.resume(w(2), None, None);
    assert_eq!(outcome, ResumeOutcome::InPlace);
    assert_eq!(ms.windows_dfs(), prima);
}

#[test]
fn d8_slot_non_identificabile_fallback() {
    let mut ms = ms_with(3);
    ms.suspend(w(2));
    // Nel frattempo il master viene chiuso → la lista cambia lunghezza.
    ms.close(w(3));
    // Lo slot non è più identificabile: il rientro segue new_placement (diventa master).
    let outcome = ms.resume(w(2), None, None);
    assert_eq!(outcome, ResumeOutcome::NewLeaf);
    assert!(ms.contains(w(2)));
}

#[test]
fn d8_rientro_finestra_gia_presente_e_noop() {
    let mut ms = ms_with(2);
    let outcome = ms.resume(w(1), None, None);
    assert_eq!(outcome, ResumeOutcome::InPlace);
    assert_eq!(ms.windows_dfs().len(), 2);
}

#[test]
fn geometria_finestre_assegnate_senza_sovrapposizione() {
    let ms = ms_with(4);
    let rects = ms.compute_rects(area(), &gaps());
    assert_eq!(rects.len(), 4);
    // Nessuna sovrapposizione a coppie.
    for (i, (_, a)) in rects.iter().enumerate() {
        for (_, b) in rects.iter().skip(i + 1) {
            let overlap = a.x < b.x + b.width as i32
                && b.x < a.x + a.width as i32
                && a.y < b.y + b.height as i32
                && b.y < a.y + a.height as i32;
            assert!(!overlap, "sovapposizione tra rettangolo {i} e un altro");
        }
    }
}

#[test]
fn stack_top_non_tocca_il_master() {
    let mut ms = MasterStack::new(MasterStackConfig {
        new_placement: NewWindowPlacement::StackTop,
        ..MasterStackConfig::default()
    });
    ms.insert(w(1));
    ms.insert(w(2));
    ms.insert(w(3));
    assert_eq!(ms.master(), Some(w(1)));
    assert_eq!(ms.stack(), &[w(3), w(2)]);
}

#[test]
fn insert_duplicato_rifiutato() {
    let mut ms = ms_with(2);
    assert!(!ms.insert(w(1)));
    assert_eq!(ms.windows_dfs().len(), 2);
}
