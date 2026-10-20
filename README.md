# Ktile — documentazione progettuale

Questa cartella contiene la documentazione progettuale per **Ktile**, tiler X11 cooperante con Muffin/Cinnamon (Linux Mint, sessione X11), riscrittura moderna ispirata all'esperienza Hyprland (layout dwindle a suddivisioni binarie e master-stack). Ktile è un client X11 esterno: **non** è un compositor e non sostituisce Muffin.

Questa versione (2026-10-20) aggiorna `docs/ARCHITECTURE.md` e `docs/ROADMAP.md` contestualmente al completamento della milestone M3 del layout core: `src/layout/tree.rs`, `dwindle.rs` e `master_stack.rs` ripristinati, con test `tests/layout_tree.rs` e `tests/layout_master.rs` allineati ai fixture geometrici. Riflette le decisioni di progetto chiuse in fase di analisi (registro decisioni D1–D9 nel rapporto di analisi, sezione 7.1). `docs/ARCHITECTURE.md` e `docs/ROADMAP.md` restano la fonte di verità tracciata del repository, aggiornata contestualmente a ogni modifica (regola operativa permanente, ADR-13).

## Documenti (fonte di verità)
- `docs/ARCHITECTURE.md` — mappa concettuale, stack, albero directory, tabella moduli con stato, flusso dati. **Da aggiornare contestualmente a ogni decisione/cambio struttura.**
- `docs/ROADMAP.md` — fasi (MVP/Fase 2/Optimization), checklist task M0–M8, registro ADR con date e motivazioni. **Da aggiornare contestualmente a ogni task completato.**
- `docs/spec/01-project-overview.md` — scopo, obiettivi, fuori scope, direzione UX.
- `docs/spec/02-architecture-rust-x11rb.md` — architettura e stack: **backend X11 scelto: x11rb** (decisione presa in analisi).
- `docs/research/03-ewmh-muffin-compatibility.md` — ricerca e compatibilità EWMH/Muffin, protocollo di verifica su Mint.
- `docs/spec/04-functional-specs-dwindle-master.md` — specifiche funzionali: layout core **dwindle e master-stack**, configurazione TOML, CLI, eleggibilità.
- `docs/research/05-roadmap-m0-m8.md` — roadmap a milestone M0–M8 con gate, template per task di coding.
- `docs/milestones/environment-baseline.md` — baseline M0 raccolta sul target.

## Decisioni chiuse (2026-10-17)
| # | Decisione | Esito |
|---|---|---|
| D1 | Nome progetto/app | **Ktile** (binario `ktile`, config `~/.config/ktile/config.toml`) |
| D2 | Chiave gap esterno | unica chiave **`outer`** (coppia `inner`/`outer`) |
| D3 | Ordine finestre | default `stable`; focus solo per la foglia target in dwindle, via `dwindle.split_target = "active"\|"mru"` (default `"active"`) |
| D4 | "Nessuna finestra eleggibile" | exit **0** + log informativo (successo operativo) |
| D5 | Layout core | **dwindle + master-stack**; vertical/horizontal/grid solo come test del motore geometrico |
| D6 | Exit 6 | rifiuto solo per EWMH essenziale assente; nessuna whitelist di nomi WM |
| D7 | Master-stack `new_status` | default **`slave`** |
| D8 | Ripristino finestre sospese | posizione originaria se lo slot è identificabile, altrimenti nuova foglia |
| D9 | Finestre sticky | escluse nella v1; rivalutazione obbligatoria in M7 |

## Avvertenze
- La documentazione è una base progettuale verificata rispetto all'analisi del Cortile originale, **non** un certificato di compatibilità Muffin: le affermazioni specifiche su Muffin devono essere verificate sulla versione target seguendo il protocollo in `docs/research/03-ewmh-muffin-compatibility.md`.
- Il repository applicativo è disponibile nel branch `feature` di `github.com/D3M-Sudo/ktile`: i file `src/layout/tree.rs`, `dwindle.rs`, `master_stack.rs`, i test `tests/layout_tree.rs`/`tests/layout_master.rs` e i fixture sono già in posto; la `docs/` di questo repository è la documentazione d'insieme.
- I nomi dei file riflettono le decisioni prese: architettura Rust con backend `x11rb` (doc 02), specifiche dei layout core dwindle/master (doc 04), roadmap a milestone M0–M8 (doc 05).
