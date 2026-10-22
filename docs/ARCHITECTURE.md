# ARCHITECTURE — Ktile

> Fonte di verità architetturale del progetto. **Regola operativa permanente:** ogni decisione architetturale, nuova funzione, cambio di stato di un modulo o riorganizzazione dei file aggiorna **contestualmente** questo file e/o `ROADMAP.md` (entrambi in `docs/`). Ogni proposta di codice include l'aggiornamento corrispondente.
> Stato redazione: 2026-10-22 — M1, M2 e M4 implementati e verificati in cloud; M3 implementata nel codice con test dedicati, ma la copertura ≥80% è in verifica CI; **M5 applicazione manuale implementata** con opt-in esplicito `--allow-apply`, gate di sicurezza live, richieste EWMH frame-aware, readback e report delle deviazioni. `cargo fmt --check`, Clippy, build, unit/integration test X11 e integrazione applier Xvfb/Openbox sono verdi su GitHub Actions. Restano aperti il test deterministico della race di distruzione finestra e l'accettazione manuale su Cinnamon/Muffin reale (vedi `ROADMAP.md`); non dichiarare superato il gate del target prima di quel test.

## 1. Mappa concettuale e stack tecnologico

Ktile è un **tiler X11 esterno, cooperante con Muffin** (Linux Mint Cinnamon, sessione X11). Non è un compositor e non sostituisce il window manager: Muffin resta autorevole su focus, stacking, decorazioni e workspace. Il riferimento UX è Hyprland (dwindle a suddivisioni binarie, master-stack, gaps), replicato in forma deterministica e senza effetti grafici.

```text
┌─────────────────────────────────────────────────────────────┐
│  Sessione X11 / Muffin (autorevole: focus, stacking, decor.) │
└──────────────▲──────────────────────────────────────────────┘
               │ EWMH/ICCCM (lettura + sole richieste geometriche)
┌──────────────┴──────────────────────────────────────────────┐
│  ktile (client X11, Rust)                                   │
│                                                             │
│  CLI/config ──► daemon (event loop, shutdown)               │
│                     │ DomainEvent                            │
│              ┌──────┴─────────┐                             │
│              │ state/         │  registry, eligibility,      │
│              │ workspaces     │  reconciliation              │
│              └──────▲─────────┘                             │
│                     │ snapshot di dominio                    │
│  x11/ ──────────────┤ connection, ewmh, icccm (read),       │
│  (adapter)          │ monitor (RandR/strut), events         │
│              ┌──────┴─────────┐                             │
│              │ layout/        │  MOTORE PURO — nessuna      │
│              │ geometry/tree/ │  connessione X11            │
│              │ dwindle/master │                             │
│              └──────▲─────────┘                             │
│                     │ Plan { moves }                         │
│              x11/applier.rs (EWMH gate + readback + BadWindow) │
└─────────────────────────────────────────────────────────────┘
```

**Stack:** Rust (stable recente 1.94.0) · backend X11 **`x11rb`** (unico, feature randr) · config TOML (`serde`+`toml`) · CLI (`clap`) · errori (`thiserror`) · test nativi `cargo test` (unitari + integrazione X11 con Xvfb+openbox).

**Policy tool di sviluppo (ADR-14 / ADR-15 / ADR-16, 2026-10-19/20):** toolchain Rust e ambiente X11 di test gestiti in ambiente cloud sandbox e in CI GitHub Actions (`.github/workflows/ci.yml`); toolchain user-level su host solo se già installata/autorizzata (amendment ADR-15). **Regola operativa per agenti: prima scelta sempre la CI (`gh run`, check PR) — mai proporre installazioni di sistema; `cargo` su host solo come fallback esplicito.**

## 2. Albero delle directory

Stato verificato sul ramo `feature-complete-roadmap` (2026-10-22, dopo M5 CI verde).

```text
<repo>/
├── AGENTS.md
├── README.md
├── Cargo.toml, Cargo.lock
├── .github/workflows/ci.yml
├── docs/
│   ├── ARCHITECTURE.md, ROADMAP.md
│   ├── handover/HANDOVER.md
│   ├── spec/ (01 overview, 02 architecture, 04 layouts, CLI matrix, config example)
│   ├── research/ (03 EWMH/Muffin, 05 milestone plan)
│   └── milestones/ (environment baseline, M1 findings/residual protocol, geometry fixtures)
├── src/
│   ├── lib.rs, main.rs, cli.rs, config.rs, errors.rs
│   ├── x11/ (connection, events, ewmh, icccm, monitor, applier)
│   ├── state/ (eligibility, reconciliation, registry, workspace)
│   └── layout/ (dwindle, geometry, master_stack, plan, tree)
└── tests/
    ├── applier_integration.rs, cli_tests.rs, config_validation.rs
    ├── eligibility.rs, exit_codes.rs, geometry.rs
    ├── layout_master.rs, layout_tree.rs, x11_integration.rs
```

## 3. Tabella dei moduli

| Path/File | Responsabilità | Funzioni principali | Stato |
|---|---|---|---|
| `src/main.rs` | Entry point, wiring, codici d'uscita | `main`, mappatura errori→exit | ✅ Completato (M1) |
| `src/cli.rs` | Argomenti CLI | parsing, `--version`, `has_display` | ✅ Implementato (M1/M4) |
| `src/config.rs` | Config TOML validata | default, validazione, limiti (gap, enum, regex) | ✅ Implementato (M4); percorso predefinito da riconciliare con D1/docs |
| `src/errors.rs` | Errori tipizzati | `KtileError`, conversione exit code (2,4,5,6) | ✅ Completato (M1) |
| `src/daemon.rs` | Event loop e shutdown | — | Non presente; pianificato (M6) |
| `src/x11/connection.rs` | Connessione X11 | `connect`, `EwmhAtoms::new` | ✅ Completato (M1) |
| `src/x11/events.rs` | Selezione e traduzione eventi | `DomainEvent`, `select_root_events` | ✅ Implementato (M1) |
| `src/x11/ewmh.rs` | Lettura proprietà EWMH | `verify_wm`, `client_list` | ✅ Completato (M1) |
| `src/x11/icccm.rs` | Lettura metadati ICCCM | `WM_CLASS`, `WM_TRANSIENT_FOR`, `WM_NORMAL_HINTS` (read-only) | ✅ Implementato (M4/M5) |
| `src/x11/monitor.rs` | Monitor e aree | `read_workspace_area` (RandR + strut/workarea) | ✅ Completato (M4); residuo Muffin locale aperto |
| `src/x11/applier.rs` | Applicazione geometrie | `_NET_MOVERESIZE_WINDOW`, gate live, frame-aware readback, report deviazioni, no retry | Implementato (M5); gate Cinnamon/Muffin reale ancora aperto |
| `src/state/registry.rs` | Stato finestre | `WindowId`, `WindowState` | ✅ Implementato (M1) |
| `src/state/eligibility.rs` | Policy di esclusione | `is_eligible`, `ExcludeConfig` | ✅ Completato (M4) |
| `src/state/workspace.rs` | Modelli workspace e monitor | tipi di dominio workspace/area | Implementato; orchestrazione incrementale ancora pianificata (M6/M7) |
| `src/state/reconciliation.rs` | Diff stato↔X11 | ricalcolo selettivo, idempotenza; M4 `reconcile_read_only` count-based (sequenza standard di inserimento, nessuna pianificazione su albero) | ✅ Read-only M4; piano incrementale su albero con `split_target=active` atteso in M6 (daemon) |
| `src/layout/geometry.rs` | Rettangoli e gap | `Rect`, `Gaps`, `compute_dwindle`, `compute_master_stack` | ✅ Completato (M2) |
| `src/layout/tree.rs` | Albero binario | insert/remove/promote (dwindle) | ✅ Completato (M3) |
| `src/layout/dwindle.rs` | Policy dwindle | split_mode/target/ratio, fallback minimi | ✅ Completato (M3) |
| `src/layout/master_stack.rs` | Policy master-stack | `new_status`, promozione, `mfact` | ✅ Completato (M3) |
| `src/layout/plan.rs` | Piano geometrico | `Plan`, `WindowMove`, `diff`, idempotenza | ✅ Completato (M2) |
| `docs/` | Documentazione | fonte di verità progettuale | In continuo aggiornamento |



> **Drift documentale aperto (non risolto arbitrariamente):** D1/README e le specifiche indicano `~/.config/ktile/config.toml`, mentre `src/main.rs` cerca attualmente `~/.config/ktile/ktile.toml` (oltre a `./ktile.toml`). Prima di cambiare codice o specifiche, decidere esplicitamente quale percorso è il contratto desiderato; fino ad allora questa divergenza è intenzionale da investigare, non un fatto da “correggere” solo in documentazione.

## 4. Flusso dei dati e architettura logica

1. **Bootstrap:** `cli` → `config` → connessione X11 (`x11/connection`) → sanity check EWMH (`_NET_SUPPORTING_WM_CHECK`, exit 6 se essenziale assente — D6) → snapshot client/workspace/monitor (`x11/ewmh`, `x11/monitor`) → stato iniziale (`state/`).
2. **Ciclo di vita:** eventi X11 (`x11/events`) → `DomainEvent` → aggiornamento stato (`state/`) → **solo se lo stato cambia in modo rilevante** → piano (`layout/`, motore puro) → diff con stato atteso → richieste controllate (`x11/applier`).
   - **Nota design M4 (PR #2):** `reconcile_read_only` usa pianificazione count-based (sequenza standard di inserimento: `eligible_windows.len()` → `compute_dwindle`/`compute_master_stack` → zip in ordine snapshot). Il piano incrementale basato sull'albero persistente con `split_target=active` è atteso in M6 (daemon); nessuna modifica codice in M4.
3. **Sicurezza del flusso:** i ConfigureNotify propri vengono consumati dal filtro anti-feedback; `BadWindow` diventa `WindowGone`; piano identico allo stato atteso → zero richieste (idempotenza); max 3 ricalcoli per burst (anti-livelock).
4. **Confini invarianti:** `layout/` non conosce X11; `state/` non parla col server; `applier` è l'unico scrittore di geometrie; hint ICCCM mai riscritti (su Muffin/Mutter mai `WM_NORMAL_HINTS`, vincolo da analisi Cortile).
