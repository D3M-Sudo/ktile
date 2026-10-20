// Test M3: albero persistente, Dwindle e selezione foglia — senza X11 (solo modello puro).

use ktile::layout::dwindle::{Dwindle, DwindleConfig, InsertOutcome, SplitTarget, MIN_SIDE};
use ktile::layout::geometry::{compute_dwindle, Gaps, Rect, SplitMode};
use ktile::layout::tree::{SplitDir, Tree};
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

fn dwindle_with(n: u32) -> Dwindle {
    let mut d = Dwindle::new(DwindleConfig::default());
    for i in 1..=n {
        d.insert(w(i), Some(w(i)), None, area(), &gaps());
    }
    d
}

#[test]
fn tree_prima_finestra_foglia_intera() {
    let mut t = Tree::new();
    assert!(t.is_empty());
    assert!(t.insert_split(w(1), w(1), SplitDir::Horizontal, 500));
    assert_eq!(t.windows_dfs(), vec![w(1)]);
    assert_eq!(t.len(), 1);
    assert!(t.contains(w(1)));
}

#[test]
fn tree_seconda_finestra_split() {
    let mut t = Tree::new();
    t.insert_split(w(1), w(1), SplitDir::Horizontal, 500);
    t.insert_split(w(2), w(1), SplitDir::Horizontal, 500);
    // Ordine DFS stabile: prima contenuto preesistente, poi la nuova finestra in `second`.
    assert_eq!(t.windows_dfs(), vec![w(1), w(2)]);
}

#[test]
fn tree_insert_target_mancante_fallisce() {
    let mut t = Tree::new();
    t.insert_split(w(1), w(1), SplitDir::Horizontal, 500);
    assert!(!t.insert_split(w(9), w(999), SplitDir::Horizontal, 500));
    assert_eq!(t.windows_dfs(), vec![w(1)]);
}

#[test]
fn tree_ratio_clamp_250_750() {
    let mut t = Tree::new();
    t.insert_split(w(1), w(1), SplitDir::Horizontal, 500);
    // ratio fuori range va clampato (nessun panic).
    assert!(t.insert_split(w(2), w(1), SplitDir::Horizontal, 900));
    assert!(t.insert_split(w(3), w(1), SplitDir::Vertical, 100));
    assert_eq!(t.windows_dfs().len(), 3);
}

#[test]
fn leaf_for_attiva_vince_sulla_mru() {
    let mut t = Tree::new();
    t.insert_split(w(1), w(1), SplitDir::Horizontal, 500);
    t.insert_split(w(2), w(1), SplitDir::Horizontal, 500);
    // MRU = w2 (ultima inserita), attiva = w1 → vince l'attiva (D3).
    assert_eq!(t.leaf_for(Some(w(1)), Some(w(2))), Some(w(1)));
    // Nessuna attiva in tiling → MRU.
    assert_eq!(t.leaf_for(Some(w(999)), Some(w(2))), Some(w(2)));
    // Né attiva né MRU valide → prima finestra DFS.
    assert_eq!(t.leaf_for(Some(w(999)), Some(w(888))), Some(w(1)));
}

#[test]
fn mru_head_aggiornato_all_inserimento() {
    let mut t = Tree::new();
    t.insert_split(w(1), w(1), SplitDir::Horizontal, 500);
    t.insert_split(w(2), w(1), SplitDir::Horizontal, 500);
    assert_eq!(t.mru_head(), Some(w(2)));
    t.mark_used(w(1));
    assert_eq!(t.mru_head(), Some(w(1)));
}

#[test]
fn dwindle_direzione_lato_piu_lungo() {
    // Area 1920x1040: la prima finestra occupa tutta l'area utile.
    let d = dwindle_with(1);
    let rects = d.compute_rects(area(), &gaps());
    assert_eq!(rects.len(), 1);
    let (_, r) = rects[0];
    assert_eq!(
        r,
        Rect {
            x: 8,
            y: 8,
            width: 1904,
            height: 1024
        }
    );
}

#[test]
fn dwindle_conferma_fixture_m2_n2() {
    // La pipeline nuova deve riprodurre la fixture M2 `dwindle-n2` (niente regressioni).
    let d = dwindle_with(2);
    let got: Vec<Rect> = d
        .compute_rects(area(), &gaps())
        .into_iter()
        .map(|(_, r)| r)
        .collect();
    let expected = compute_dwindle(area(), &gaps(), 2, SplitMode::LongestSide, 0.5);
    assert_eq!(got, expected);
}

#[test]
fn dwindle_conferma_fixture_m2_n4_longest_side() {
    // Con longest-side la pipeline nuova coincide con M2 anche a n=4.
    let d = dwindle_with(4);
    let got: Vec<Rect> = d
        .compute_rects(area(), &gaps())
        .into_iter()
        .map(|(_, r)| r)
        .collect();
    let expected = compute_dwindle(area(), &gaps(), 4, SplitMode::LongestSide, 0.5);
    assert_eq!(got, expected);
}

#[test]
fn dwindle_alternating_opposto_al_padre() {
    // Con alternating il secondo split deve essere perpendicolare al padre.
    let mut d = Dwindle::new(DwindleConfig {
        split_mode: SplitMode::Alternating,
        ..DwindleConfig::default()
    });
    for i in 1..=3 {
        d.insert(w(i), Some(w(i)), None, area(), &gaps());
    }
    let rects = d.compute_rects(area(), &gaps());
    assert_eq!(rects.len(), 3);
    // Il secondo split (che separa w2 e w3) deve dividere in verticale: w2 e w3 condividono x.
    let (_, r2) = rects[1];
    let (_, r3) = rects[2];
    assert_eq!(
        r2.x, r3.x,
        "alternating: w2 e w3 devono impilarsi verticalmente"
    );
}

#[test]
fn dwindle_target_mru_config() {
    let mut d = Dwindle::new(DwindleConfig {
        split_target: SplitTarget::Mru,
        ..DwindleConfig::default()
    });
    for i in 1..=3 {
        d.insert(w(i), Some(w(i)), None, area(), &gaps());
    }
    // MRU = w3 (ultima inserita): il quarto va vicino a w3, non all'attiva w1.
    d.insert(w(4), Some(w(1)), Some(w(3)), area(), &gaps());
    let dfs = d.tree().windows_dfs();
    // w4 deve stare appena dopo w3 (second della sua foglia).
    let pos3 = dfs.iter().position(|&x| x == w(3)).unwrap();
    let pos4 = dfs.iter().position(|&x| x == w(4)).unwrap();
    assert_eq!(pos4, pos3 + 1);
}

#[test]
fn dwindle_chiusura_promuove_il_fratello() {
    let mut d = dwindle_with(3);
    let prima = d.compute_rects(area(), &gaps());
    assert!(d.close(w(2)));
    let dopo = d.compute_rects(area(), &gaps());
    // w2 esce; restano due finestre.
    assert_eq!(dopo.len(), 2);
    assert!(!d.tree().contains(w(2)));
    let get = |rects: &[(WindowId, Rect)], id: u32| -> Rect {
        rects
            .iter()
            .find(|(w, _)| w.0 == id)
            .map(|(_, r)| *r)
            .expect("finestra attesa")
    };
    let area_of = |r: &Rect| (r.width as u64) * (r.height as u64);
    // Promozione: la finestra superiore del ramo (w3) eredita l'intera colonna destra,
    // quindi la sua area cresce rispetto a prima (prima condivideva il ramo con w2).
    assert!(area_of(&get(&dopo, 3)) > area_of(&get(&prima, 3)));
    // Nessuna sovrapposizione dopo la promozione.
    let a = get(&dopo, 1);
    let b = get(&dopo, 3);
    let overlap = a.x < b.x + b.width as i32
        && b.x < a.x + a.width as i32
        && a.y < b.y + b.height as i32
        && b.y < a.y + a.height as i32;
    assert!(!overlap);
}

#[test]
fn dwindle_chiusura_master_tree_vuoto() {
    let mut d = dwindle_with(1);
    assert!(d.close(w(1)));
    assert!(d.tree().is_empty() || d.tree().len() == 0);
    assert!(d.compute_rects(area(), &gaps()).is_empty());
    // Chiusura due volte: la seconda non falliscepanic.
    assert!(!d.close(w(1)));
}

#[test]
fn dwindle_sospensione_d8_rientro_in_posizione() {
    let mut d = dwindle_with(3);
    let prima = d.compute_rects(area(), &gaps());
    assert!(d.suspend(w(2)));
    assert!(!d.tree().contains(w(2)));
    assert_eq!(d.compute_rects(area(), &gaps()).len(), 2);
    // Rientro: slot ancora identificabile (nessun altro ha preso il posto).
    let outcome = d.resume(w(2), None, None);
    assert_eq!(outcome, ktile::layout::tree::ResumeOutcome::InPlace);
    let dopo = d.compute_rects(area(), &gaps());
    assert_eq!(dopo, prima, "rientro D8 = stessa geometria di partenza");
}

#[test]
fn dwindle_d8_slot_occupato_non_possibile() {
    // Lo slot sospeso non è mai bersaglio di insert (i target sono solo finestre presenti):
    // anche con nuove finestre aperte, il rientro di w2 trova ancora il suo slot.
    let mut d = dwindle_with(3);
    d.suspend(w(2));
    d.insert(w(9), Some(w(3)), None, area(), &gaps());
    let outcome = d.resume(w(2), None, Some(w(9)));
    assert_eq!(outcome, ktile::layout::tree::ResumeOutcome::InPlace);
    assert!(d.tree().contains(w(2)));
    assert_eq!(d.tree().windows_dfs().len(), 4);
}

#[test]
fn dwindle_d8_fallback_finestra_mai_parcheggiata() {
    // Una finestra mai sospesa non ha slot: il rientro cade nel fallback (nuova foglia via MRU).
    let mut d = dwindle_with(2);
    let outcome = d.resume(w(7), None, Some(w(2)));
    assert_eq!(outcome, ktile::layout::tree::ResumeOutcome::NewLeaf);
    assert!(d.tree().contains(w(7)));
    assert_eq!(d.tree().windows_dfs().len(), 3);
    // Rientro di una finestra già presente: no-op InPlace.
    let again = d.resume(w(7), None, None);
    assert_eq!(again, ktile::layout::tree::ResumeOutcome::InPlace);
}

#[test]
fn dwindle_min_side_fallback_area_piccola() {
    // Area minuscola: dopo la prima finestra non c'è spazio per uno split >= 50px.
    let tiny = Rect {
        x: 0,
        y: 0,
        width: 60,
        height: 60,
    };
    let mut d = Dwindle::new(DwindleConfig::default());
    assert_eq!(
        d.insert(w(1), Some(w(1)), None, tiny, &gaps()),
        InsertOutcome::Tiled
    );
    // 60x60 meno outer 8*2 = 44x44: uno split darebbe ~18px < MIN_SIDE → floating.
    let outcome = d.insert(w(2), Some(w(2)), None, tiny, &gaps());
    assert_eq!(outcome, InsertOutcome::Floating);
    assert_eq!(d.tree().windows_dfs(), vec![w(1)], "w2 resta fuori dal tiling");
}

#[test]
fn dwindle_min_side_costante_50() {
    assert_eq!(MIN_SIDE, 50);
}

#[test]
fn dwindle_n4_second_sulla_foglia_mru() {
    // La quarta finestra fa split sulla foglia MRU (w3): in DFS w4 segue w3.
    let d = dwindle_with(4);
    let dfs = d.tree().windows_dfs();
    assert_eq!(dfs.len(), 4);
    // w4 e' in `second` della foglia di w3: l'ordine DFS deve vederli adiacenti.
    let pos3 = dfs.iter().position(|&x| x == w(3)).unwrap();
    let pos4 = dfs.iter().position(|&x| x == w(4)).unwrap();
    assert_eq!(pos4, pos3 + 1, "w4 deve essere appena dopo w3 in DFS");
}
