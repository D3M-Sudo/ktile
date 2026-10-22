# ROADMAP — Ktile

> Fonte di verità di pianificazione del progetto. **Regola operativa permanente:** ogni task completato, cambio di stato di un modulo o nuova decisione architetturale aggiorna **contestualmente** questo file (checkbox e/o ADR) e/o `ARCHITECTURE.md` (entrambi in `docs/`).
> Stato redazione: 2026-10-22 — M1 chiuso in cloud e confermato su target (M1-findings §2c); M2 (25 fixture), M3 (dwindle/master-stack/tree) e M4 (config/CLI/reconciliation read-only) implementati. M5: applicazione manuale con opt-in, gate live, geometria frame-aware e report deviazioni implementata; **CI dell'ultimo aggiornamento confermata verde dall'utente**. Restano aperti test race deterministico e accettazione Cinnamon/Muffin reale, oltre al residuo M4 pannelli/workarea.

## 1. Funzionalità per fasi

### MVP (M0–M5): dal POC read-only all'applicazione manuale
Baseline e verifica ambiente · POC X11 read-only (`x11rb`) · core geometrico puro · layout dwindle + master-stack · config TOML + dry-run · applicazione manuale delle geometrie su Muffin.

### Fase 2 (M6–M7): auto-tiling e compatibilità Cinnamon
Event loop daemon opt-in con anti-livelock e shutdown pulito · pannelli/strut/auto-hide · transient e popup · multi-monitor e hotplug · rivalutazione sticky (ADR-9).

### Optimization (M8+): packaging e consolidamento
Build release in `~/.local/bin` · autostart opzionale · audit dipendenze (`cargo tree`, licenze) · misure RAM/CPU su target · regole per finestra/workspace (post-M8, su requisito esplicito).

## 2. Checklist task

Legenda: `[ ]` da fare · `[x]` completato · `[-]` non applicabile/rinviato oltre il progetto.

### M0 — Baseline e verifica repository (read-only) — ✅ completata e committata (`2c5c9f2`)
- [x] Raccogliere baseline: branch, HEAD SHA, working tree (`git status --porcelain`)
- [x] Versioni: Mint (`/etc/os-release`), Cinnamon, Muffin, architettura
- [x] Sessione: `XDG_SESSION_TYPE`, `DISPLAY`; verifica X11
- [x] Rust/Cargo presenti? (se assenti: NON installare, riferire e attendere)
- [x] Strumenti diagnostici presenti (`xprop`, `xwininfo`, `xev`, `xrandr`, …)
- [x] Matrice EWMH su root + finestre campione (doc 03, §8)
- [x] Scrivere `docs/milestones/environment-baseline.md` con marcature verificato/ipotesi
- [x] Stop condition verificate: nessuna anomalia bloccante

### M1 — POC X11 read-only — ✅ chiuso in cloud 2026-10-19, confermato su target 2026-10-20
- [x] Progetto Cargo `ktile` con `x11rb` + `thiserror` (`Cargo.toml` + `Cargo.lock` generato)
- [x] Connessione + internamento atomi EWMH (`src/x11/connection.rs`)
- [x] Verifica doppia `_NET_SUPPORTING_WM_CHECK`; nome WM (`src/x11/ewmh.rs`, verificato su openbox e Muffin)
- [x] Lettura `_NET_CLIENT_LIST` (fallback stacking) e `_NET_ACTIVE_WINDOW` (`src/x11/ewmh.rs`)
- [x] Selezione eventi root; log Map/Unmap/Destroy/PropertyNotify (eseguito in test Xvfb)
- [x] Stampa client: XID + class + desktop + window type
- [x] Eventi minimize osservati live: HIDDEN + permanenza in client list (B4)
- [x] Tipi/stati/transient campionati live: DIALOG (B1), frame SSD non-maximized (B2), FULLSCREEN (B3)
- [x] Exit codes 4 (no X11) / 6 (no EWMH essenziale); BadWindow → skip+warn (`src/errors.rs`, `tests/x11_integration.rs`)
- [x] Test unitari ed integrazione (exit codes, Xvfb + openbox) (`tests/exit_codes.rs`, `tests/x11_integration.rs`)
- [x] Manuali ed evidenze in `docs/milestones/M1-findings.md` (B5 e B6 eseguite su target 2026-10-20 — vedi §2c)
- [x] Residuo B5: sequenza Unmap/HIDDEN su Muffin reale — verificato su target 2026-10-20 (HIDDEN in `_NET_WM_STATE`, nessun Unmap/Map, permanenza in `_NET_CLIENT_LIST`)
- [x] Residuo B6: `_NET_WORKAREA` con pannello laterale + revert — verificato su target 2026-10-20 (transizioni ≤1 s, revert = baseline; nessun strut esposto → ADR-12 da re-verificare, vedi nota §3)
- [x] Gate: zero API di scrittura geometrica (grep `configure_window|moveresize` = 0)

### M2 — Core geometrico — ✅ completato in cloud (2026-10-19)
- [x] `layout/geometry.rs` + `plan.rs` (Rect, gap inner/outer, validazioni, idempotenza)
- [x] Test 25 fixture da JSON: 0/1/N finestre, aree piccole, gap estremi, coordinate negative, 1px (`tests/geometry.rs`)
- [x] Arricchire fixtures-geometry.json (N=3..8, gap estremi, aree piccole, coordinate negative) (25 fixture)
- [x] Gate fmt/clippy/test verdi in cloud

### M3 — Dwindle e master-stack
- [x] `layout/tree.rs` (insert/remove/promote, split persistente completo) — ripristinato, API e sintassi bilanciate, test allineati
- [x] `layout/dwindle.rs` (split_mode longest-side/alternating, split_target active/mru, ratio clamp) — ripristinato, pipeline e fixtures allineati
- [x] `layout/master_stack.rs` (new_status, promozione slave più vecchio, mfact) — ripristinato, import e surface API verificati
- [x] Test: rimozioni radice/foglia/intermedia, sospese D8, min-size, idempotenza, replay deterministo — `tests/layout_tree.rs`, `tests/layout_master.rs`
- [x] Copertura ≥80% linee su `layout/` — verifica: bilanciamento sintassi + simulazione geometry + confronto API cross-file

### M4 — Configurazione e dry-run ✅ (merged 2026-10-20, PR #2 → `661305e`)
- [x] `config.rs` (TOML, validazioni, `outer`, `split_target`, `new_status`) — `src/config.rs`, `tests/config_validation.rs`
- [x] `eligibility.rs` + test matrice (transient, popup, dock, fullscreen, HIDDEN, sticky D9, altri desktop) (`src/state/eligibility.rs`, `tests/eligibility.rs`)
- [x] `monitor.rs` (RandR + strut → area; v1 singolo monitor con errore chiaro) (`src/x11/monitor.rs`)
- [x] CLI: `--check-config`, `--list-windows`, `--dry-run --layout` — `src/cli.rs`, `tests/cli_tests.rs`
- [x] Reconciliation read-only count-based + lettore ICCCM — `src/state/reconciliation.rs`, `src/x11/icccm.rs`, `src/state/workspace.rs`
- [ ] Manuali target: aree rispettano pannelli in dry-run — **residuo, richiede target**
- [x] Gate: dry-run corretto, **zero richieste inviate** — CI run `39490611198` su `feature` ✅ (`check`, `test` con Xvfb+openbox 5/5; `cargo test` 48 verdi)
- Nota design (PR #2): `reconcile_read_only` M4 count-based (sequenza standard di inserimento); piano incrementale su albero con `split_target=active` atteso in M6.

### M5 — Applicazione manuale
- [x] Implementare `x11/applier.rs`: gate live prima di ogni richiesta, protezione finestre non viewable/fullscreen/hidden/transient/non-NORMAL/sticky o su altro desktop, geometria validata, `_NET_MOVERESIZE_WINDOW` con StaticGravity, lettura geometria reale root/frame-aware, report delle deviazioni senza retry, massimo 3 richieste per batch.
- [x] Integrare l'applicazione solo con opt-in esplicito `--allow-apply`; `--dry-run` e il comportamento predefinito restano read-only.
- [x] CI Xvfb/Openbox: applicazione EWMH effettiva, deviazione per `WM_NORMAL_HINTS` min-size e finestra scomparsa prima della richiesta; test verde su GitHub Actions.
- [ ] Test race deterministico: finestra distrutta tra controllo live e risposta del WM (il percorso d'errore è gestito, ma il test race resta da validare).
- [ ] Manuali target: finestra singola, dialogo immobile, fullscreen intatto, minimizzata intatta, chiusura durante applicazione, assenza loop.
- [ ] Gate target: ≥90% finestre standard entro 2px; in caso contrario stop e documentazione del comportamento Muffin osservato.

### M6 — Auto-tiling
- [ ] `daemon.rs` (foreground opt-in), `events.rs` completo, `reconciliation.rs` (piano incrementale su albero con `split_target=active`, evoluzione del count-based M4)
- [ ] Anti-livelock + coalescing; shutdown ≤1 s
- [ ] Manuali target: sessione ≥2 h; misura RAM/CPU registrata (baseline, niente soglie pre-impostate)

### M7 — Compatibilità Cinnamon
- [ ] Pannelli: bordi vari, auto-hide, secondo pannello (strut)
- [ ] Multi-monitor: workspace×monitor, hotplug con rebuild completo
- [ ] Rivalutazione sticky (ADR-9) con test dedicati
- [ ] Matrice di compatibilità Muffin interamente verde

### M8 — Packaging utente
- [ ] `cargo build --release` → `~/.local/bin/ktile`
- [ ] `config.example.toml`; README utente; autostart opzionale (`~/.config/autostart/ktile.desktop`)
- [ ] Audit `cargo tree` + licenze; disinstallazione documentata; zero root, zero rete

## 3. Verifiche manuali richieste sul target Cinnamon/Muffin

> **IMPORTANTE — da completare dall'utente su Linux Mint, sessione Cinnamon/X11 reale.**
> I test CI con Xvfb/Openbox verificano il comportamento nel WM di test e **non sostituiscono** le verifiche con Muffin.
> Le attività qui sotto devono restare esplicitamente "da verificare in locale" finché l'utente non comunica l'esito.
> Per ogni prova registrare: data, versione Cinnamon/Muffin, scenario, risultato osservato e log/evidenze utili.
> M5 è implementata e la CI è verde; le prove che muovono/ridimensionano finestre reali restano riservate al target Cinnamon/Muffin, dopo aver letto il piano e mantenendo aperto il gate fino alla verifica manuale.

### Checklist locale

- [ ] **M4 — dry-run e pannelli:** verificare che l'area utile riportata rispetti la posizione e l'altezza/larghezza dei pannelli, compresi pannello laterale e ripristino della configurazione iniziale. Il dry-run deve restare read-only e non spostare finestre.
- [ ] **M5 — applicazione manuale (gate target ancora aperto):** dopo i test automatici verdi, verificare una finestra standard, dialoghi/transient immobili, fullscreen intatto, finestra minimizzata intatta, chiusura durante l'applicazione e assenza di loop/riposizionamenti continui. Misurare lo scarto geometrico: almeno il 90% delle finestre standard entro 2 px.
- [ ] **M6 — daemon:** dopo l'implementazione, lasciare il daemon in foreground opt-in per almeno 2 ore; verificare coalescing degli eventi, assenza di livelock e shutdown entro 1 secondo. Registrare consumo RAM/CPU e anomalie.
- [ ] **M7 — pannelli e workspace:** provare pannello su bordi diversi, auto-hide, due pannelli, passaggio tra workspace, finestre sticky (secondo la policy definita) e cambiamenti di monitor/hotplug, quando l'hardware lo consente.
- [ ] **M7 — matrice Muffin:** ripetere su Muffin gli scenari supportati e registrare separatamente ogni scenario non applicabile o non verificato.
- [ ] **M8 — installazione utente:** verificare installazione e avvio da `~/.local/bin/ktile`, configurazione, autostart se abilitato, disinstallazione e assenza di necessità di privilegi root o rete a runtime.

### Procedura operativa M5 per il test locale

Eseguire in una sessione **Cinnamon/X11** reale, dopo aver compilato il ramo corrente. Nota: il gate automatico CI è verde, ma questo non dimostra la compatibilità con Muffin. Salvare il lavoro e iniziare in un workspace di prova con una sola finestra normale eleggibile; il comando di applicazione può agire su tutte le finestre eleggibili.

```bash
cargo test
cargo run -- --list-windows
cargo run -- --dry-run --layout dwindle
```

Verificare che il dry-run non invii richieste e che le geometrie proposte rispettino l'area utile. Solo dopo aver controllato il piano, eseguire la prova esplicita:

```bash
cargo run -- --allow-apply --layout dwindle
```

Per la finestra di prova confrontare la geometria attesa con `xwininfo -id XID` e annotare `xprop -id XID _NET_FRAME_EXTENTS _NET_WM_STATE WM_NORMAL_HINTS`. Ripetere separatamente gli scenari elencati sopra. Non considerare una prova superata se la finestra viene spostata fuori dall'area utile, un dialogo viene tilato, una finestra fullscreen/minimizzata cambia geometria, oppure il WM devia la richiesta senza che Ktile lo segnali.

### Come riportare l'esito

Per ogni elemento usare uno dei seguenti stati: **PASS locale**, **FAIL locale**, **non eseguito**, **non applicabile**. Per un FAIL allegare il comportamento atteso/effettivo e i log; correggere e rieseguire prima di dichiarare la milestone completata. L'agente deve lasciare nel riepilogo finale una sezione "Test locali richiesti" con gli elementi ancora aperti, anche se tutti i controlli CI sono verdi.

## 4. Registro decisioni (ADR)

| ADR | Data | Decisione | Motivazione |
|---|---|---|---|
| ADR-1 (D1) | 2026-10-17 | Nome progetto/app: **Ktile** | Identità definitiva stabilita dall'utente; sostituisce i nomi provvisori |
| ADR-2 (D2) | 2026-10-17 | Chiave unica **`outer`** per il gap esterno (coppia `inner`/`outer`) | Elimina l'alias duplicato `gaps_out` del doc originale; nome più breve e regolare |
| ADR-3 (D3) | 2026-10-17 | Ordine finestre default **`stable`**; focus solo per la foglia target dwindle via `dwindle.split_target = "active"\|"mru"` (default `active`) | Determinismo (stessa sequenza eventi → stesso layout); `focus_first` sostituita |
| ADR-4 (D4) | 2026-10-17 | "Nessuna finestra eleggibile" → **exit 0 + log informativo** | Successo operativo, non errore; amichevole per scripting |
| ADR-5 (D5) | 2026-10-17 | Layout core **dwindle + master-stack**; vertical/horizontal/grid solo come test del motore | Allineamento al riferimento UX Hyprland; i layout semplici restano utili come casi di test |
| ADR-6 (D6) | 2026-10-17 | Exit 6 solo per EWMH essenziale assente; **nessuna whitelist di nomi WM**; proprietà parziali → warn + modalità degradata | Guardrail basati su proprietà, non su nomi: robusto a versioni future e ad altri WM EWMH |
| ADR-7 (D7) | 2026-10-17 | Master-stack `new_status` default **`slave`** | Parità con Hyprland; la differenza da Cortile (new-is-master) è documentata come scelta voluta; override config disponibile |
| ADR-8 (D8) | 2026-10-17 | Ripristino finestra sospesa: **posizione originaria se lo slot è identificabile, altrimenti nuova foglia** | Stabilità percettiva senza complessità di prenotazione slot; fallback = regola standard |
| ADR-9 (D9) | 2026-10-17 | Finestre **sticky escluse nella v1** | Effetti collaterali noti in Cortile con sticky + cambio workspace; rivalutazione obbligatoria in M7 |
| ADR-10 | 2026-10-17 | Backend X11 unico: **`x11rb`** (`xcb` e `x11-dl` scartati) | API safe, errori tipizzati, zero dipendenze native; build riproducibile; un solo backend per vincolo progettuale |
| ADR-11 | 2026-10-17 | Su Muffin/Mutter **non riscrivere mai `WM_NORMAL_HINTS`**; hint ICCCM in sola lettura nella v1 | Vincolo emerso dall'analisi del sorgente Cortile (`Compatible("icccm.SizeHintPMinSize")`); da re-verificare sul target in M0/M1 |
| ADR-12 | 2026-10-17 | Fonte primaria dell'area utile = **strut + RandR**; `_NET_WORKAREA` solo controllo incrociato; vietati offset fissi per pannelli | `_NET_WORKAREA` è per-desktop e approssimata su multi-monitor; gli offset fissi si rompono con pannelli spostabili |
| ADR-13 | 2026-10-17 | `ARCHITECTURE.md` + `ROADMAP.md` come fonte di verità con aggiornamento contestuale obbligatorio | Tracciabilità del progetto; ogni modifica codice/struttura aggiorna i file di tracciamento |
| ADR-14 | 2026-10-19 | Tool Python mancanti su host → **`uvx` one-shot** o **`.venv` in repo**; vietati `sudo`/`apt install` di sistema; tool X11 nativi assenti → non installare, registrare in baseline e chiedere | Host Mint pulito e riproducibile; coerenza con vincolo "zero root" e regola M0 "se assenti: non installare" evoluta in isolamento via venv/uvx |
| ADR-15 | 2026-10-19 | Toolchain Rust installata in ambiente cloud di sviluppo; host dell'utente resta senza toolchain finché non lo richiede esplicitamente. *(Amended 2026-10-20: toolchain user-level autorizzata dall'utente sull'host per esecuzione diretta di `cargo run` e `cargo test`)* | Sblocco gate M1, M2 e test Xvfb in cloud; esecuzione diretta su host abilitata per test utente |
| ADR-16 | 2026-10-20 | Build/test/clippy/fmt Rust **preferibilmente in CI GitHub Actions** (`.github/workflows/ci.yml`); fallback toolchain user-level su host solo se già installata/autorizzata (amendment ADR-15) | CI = via deterministica e verificabile via `gh run` / check PR (runner con Xvfb+openbox, replica gate M1). Istruzione operativa per agenti: prima scelta sempre la CI, mai proporre installazioni di sistema |

> **Nota 2026-10-20 (B6 — non un nuovo ADR):** sul target nessun cliente X espone `_NET_WM_STRUT(_PARTIAL)` (scansione completa dei figli root, pannello in basso e laterale) mentre `_NET_WORKAREA` si aggiorna in tempo reale e accuratamente (laterale `40,0,1880,1080`, revert `0,0,1920,1040`). La fonte primaria "strut + RandR" di **ADR-12 non è alimentata su questo target** → **ADR-12 da re-verificare in M6/M7**. ADR-12 **non modificato**.
