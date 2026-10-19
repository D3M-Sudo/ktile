# 05 — Roadmap di sviluppo (milestone M0–M8)

Roadmap autorizzata dal rapporto di analisi (sezione 8), allineata alle decisioni chiuse D1–D9 (2026-10-17). Ogni milestone chiude con i gate di qualità: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`. L'auto-tiling resta **opt-in** fino a M6 e disattivato di default anche dopo. Nessun commit/push salvo richiesta esplicita; niente `sudo`.

## M0 — Baseline e verifica del repository

- **Obiettivo misurabile:** baseline documentata (branch, HEAD SHA, working tree, versioni Mint/Cinnamon/Muffin, sessione X11, Rust, strumenti, esiti EWMH).
- **File ammessi:** `docs/milestones/environment-baseline.md` (unica scrittura).
- **Test:** nessuno (sola ispezione: comandi read-only sezione 2.1 e matrice EWMH del doc 03).
- **Stop condition:** working tree sporco, sessione non X11, Rust assente, repository assente → fermarsi e riferire.

## M1 — POC X11 read-only

- **Prerequisiti:** M0 ok; autorizzazione a creare il progetto Cargo.
- **Obiettivo misurabile:** `ktile` si connette, identifica il WM via `_NET_SUPPORTING_WM_CHECK`, elenca i client EWMH, osserva Map/Unmap/Destroy/PropertyNotify per 30 s; **zero richieste geometriche**; exit 4 senza DISPLAY, 6 senza EWMH essenziale.
- **File ammessi:** `Cargo.toml`, `Cargo.lock`, `.gitignore`, `src/main.rs`, `src/cli.rs`, `src/errors.rs`, `src/x11/{mod.rs,connection.rs,events.rs,ewmh.rs}`, `src/state/{mod.rs,registry.rs}` (solo modello), `tests/exit_codes.rs`, `docs/milestones/M1-findings.md`.
- **Test automatici:** unitari errori→exit code, parsing argomenti; integrazione Xvfb/Xephyr marcata `#[ignore]`.
- **Manuali (target):** avvio su Cinnamon reale, apertura/chiusura finestre osservata, Ctrl-C pulito, avvio senza DISPLAY.
- **Stop condition:** EWMH parzialmente assente su Muffin con impatto di design → riferire prima di procedere.

## M2 — Core geometrico

- **Obiettivo misurabile:** `layout/` puro con Rect, gaps (`inner`/`outer`, D2), validazione aree, invarianti (nessuna sovrapposizione, dentro l'area, ≥1px) su ≥ 40 casi.
- **File ammessi:** `src/layout/{mod.rs,geometry.rs,plan.rs}`, `tests/geometry_edge.rs`.
- **Test:** 0/1/N finestre, area < gap, gap estremi, coordinate negative, rettangoli 1px, N=128, rapporti estremi.
- **Stop condition:** emergono assunzioni che richiedono X11 (confine violato: correggere il confine).

## M3 — Dwindle e master-stack

- **Obiettivo misurabile:** albero insert/remove/promote; master-stack con promozione slave→master; policy split deterministica; replay identico della stessa sequenza di operazioni; invarianti su sequenze a seed fissi.
- **File ammessi:** `src/layout/{tree.rs,dwindle.rs,master_stack.rs}`, `tests/layout_tree.rs`, `tests/layout_master.rs`.
- **Test:** 1–64 finestre, rimozione radice/foglia/intermedia, slot sospeso (D8), min-size ICCCM simulata, idempotenza del piano.
- **Decisioni in vigore:** D3 (`stable` + `split_target=active`), D7 (`new_status=slave`), D8. Deviazione → prima proposta decisionale.
- **Stop condition:** policy che richiedono rinegoziazione delle decisioni chiuse.

## M4 — Configurazione e dry-run

- **Obiettivo misurabile:** `--check-config` valida TOML secondo doc 04; `--dry-run --layout dwindle|master-stack` stampa il piano (XID → Rect) senza inviare richieste.
- **File ammessi:** `src/config.rs`, ampliamento `src/cli.rs`, `src/state/eligibility.rs`, `src/state/reconciliation.rs` (lettura), `src/x11/icccm.rs` (lettura meta), `src/x11/monitor.rs` (RandR), `tests/eligibility.rs`, `config.example.toml`.
- **Manuali (target):** `--list-windows`/`--dry-run` su Cinnamon: esclusione di pannello/dialoghi/altri workspace; aree rispettano i pannelli (strut).
- **Stop condition:** area calcolata non rispetta i pannelli in dry-run → non passare a M5.

## M5 — Applicazione manuale

- **Prerequisiti:** M4 ok; consenso dell'utente in sessione di test; gate dry-run superato.
- **Obiettivo misurabile:** `--layout` applica il piano a una finestra di test poi all'intero workspace attivo; lettura risultato; log deviazioni; max 3 richieste per finestra.
- **File ammessi:** `src/x11/applier.rs`, estensioni `events.rs` (filtro in-flight), `tests/applier_logic.rs` (mock).
- **Manuali (target, non distruttivi):** finestra singola; dialogo immobile; fullscreen immutato; minimizzata mai ripristinata; pannello intatto; chiusura durante applicazione; assenza di loop (log stabile dopo 5 s).
- **Gate quantitativo:** ≥90% finestre standard entro tolleranza 2px, altrimenti stop e documentazione.

## M6 — Auto-tiling (event loop)

- **Obiettivo misurabile:** `--daemon` (foreground, opt-in) reagisce agli eventi con ricalcolo selettivo; CPU inattiva ≈ 0 e RAM stabile **misurate su target** (nessuna soglia pre-impostata; prima misura = baseline); shutdown su segnali ≤1 s.
- **File ammessi:** `src/daemon.rs`, `src/x11/events.rs` (completo), `src/state/reconciliation.rs` (completo), test integrazione Xvfb.
- **Manuali (target):** sessione reale ≥2 h; verifica log senza spam; misura RAM/CPU registrata.
- **Stop condition:** loop rilevati o consumo anomalo.

## M7 — Compatibilità Cinnamon

- **Obiettivo misurabile:** pannelli su bordi vari/auto-hide corretti (strut); transient/popup secondo policy; **multi-monitor** supportato (workspace×monitor, no sovrapposizioni); sticky rivalutati (D9) con test dedicato; hotplug senza crash.
- **Stop condition:** hotplug instabile → mantenere singolo monitor documentato.

## M8 — Packaging utente

- **Obiettivo misurabile:** `cargo build --release` → `~/.local/bin/ktile`; `config.example.toml`; autostart **opzionale** (`~/.config/autostart/ktile.desktop`); disinstallazione documentata; zero root; zero rete.
- **File ammessi:** `README.md` (utente), `packaging/`, eventuale `install.sh` (solo copia in home).
- **Test:** build pulita da `cargo clean`; audit `cargo tree` e licenze.

## Template per task di coding (vibe coding)

```text
Obiettivo: <un solo obiettivo misurabile>
Contesto: leggi docs/spec/01-project-overview.md, docs/spec/02-architecture-rust-x11rb.md,
docs/research/03-ewmh-muffin-compatibility.md, docs/spec/04-functional-specs-dwindle-master.md,
docs/research/05-roadmap-m0-m8.md e docs/milestones/environment-baseline.md.
Vincoli:
- non eseguire git commit/push/merge/tag né cambiare branch;
- non installare pacchetti di sistema e non usare sudo;
- non spostare finestre finché il task è read-only;
- non modificare file fuori dallo scope elencato;
- se branch/working tree o ambiente differiscono dalle assunzioni, fermati e riferisci.
File ammessi: <elenco>
File esclusi: <elenco>
Test: <comandi automatici e manuali non distruttivi>
Criteri di accettazione: <lista verificabile>
Output finale: file modificati, test eseguiti, risultati, rischi e prossima milestone.
```

## Primo task concreto

**M0 + M1**: M0 raccoglie la baseline read-only sulla macchina target (script di ispezione nel doc 03 e sezione 2.1 del rapporto) e scrive `docs/milestones/environment-baseline.md`; M1 crea il POC Cargo con il solo backend `x11rb` (dipendenze minime: `x11rb`, `thiserror`), connessione + verifica EWMH + elenco client + osservazione eventi, con exit 4/6 e nessuna API di scrittura geometrica. I due prompt operativi dettagliati (file ammessi/esclusi, test, criteri) sono già stati consegnati in sessione. Gate: osservazione ripetibile documentata in `docs/milestones/M1-findings.md` prima di autorizzare M2.
