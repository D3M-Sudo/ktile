# ARCHITECTURE — Ktile

> Fonte di verità architetturale del progetto. **Regola operativa permanente:** ogni decisione architetturale, nuova funzione, cambio di stato di un modulo o riorganizzazione dei file aggiorna **contestualmente** questo file e/o `ROADMAP.md` (entrambi in `docs/`). Ogni proposta di codice include l'aggiornamento corrispondente.
> Stato redazione: 2026-10-19 — M1 chiuso in cloud (rustc/cargo 1.94.0, build/clippy/test verdi, test Xvfb+openbox); M2 completato in cloud (25/25 fixture verdi); moduli `monitor` ed `eligibility` implementati e testati.

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
│              x11/applier.rs (gate + anti-feedback + BadWindow)│
└─────────────────────────────────────────────────────────────┘
```

**Stack:** Rust (stable recente 1.94.0) · backend X11 **`x11rb`** (unico, feature randr) · config TOML (`serde`+`toml`) · CLI (`clap`) · errori (`thiserror`) · test nativi `cargo test` (unitari + integrazione X11 con Xvfb+openbox).

**Policy tool di sviluppo (ADR-14 / ADR-15, 2026-10-19):** toolchain Rust e ambiente X11 di test gestiti in ambiente cloud sandbox; l'host dell'utente fisica resta pulito e senza modifiche non richieste.

## 2. Albero delle directory

Stato attuale (2026-10-19) — M1/M2 completati in cloud.

```text
<repo>/
├── README.md                  ← indice documentazione (root)
├── Cargo.toml, Cargo.lock     ← progetto Rust compilabile
├── docs/
│   ├── ARCHITECTURE.md        ← fonte di verità architetturale (questo file)
│   ├── ROADMAP.md             ← milestone M0–M8, checklist, ADR
│   ├── spec/                  ← specifiche di progetto
│   ├── research/              ← ricerca e protocolli di verifica
│   └── milestones/            ← evidenze per milestone
│       ├── environment-baseline.md    (M0)
│       ├── M1-findings.md             (M1/M2 risultati cloud)
│       ├── M1-residuals-user.md       (Protocollo residui macchina target)
│       └── fixtures-geometry.json     (25 fixture di test)
├── src/
│   ├── lib.rs, main.rs, cli.rs, errors.rs
│   ├── x11/  (connection, ewmh, events, monitor)
│   ├── state/ (registry, eligibility)
│   └── layout/ (mod, geometry, plan)
└── tests/
    ├── exit_codes.rs, geometry.rs, eligibility.rs, x11_integration.rs
```

## 3. Tabella dei moduli

| Path/File | Responsabilità | Funzioni principali | Stato |
|---|---|---|---|
| `src/main.rs` | Entry point, wiring, codici d'uscita | `main`, mappatura errori→exit | ✅ Completato (M1) |
| `src/cli.rs` | Argomenti CLI | parsing, `--version`, `has_display` | ✅ Implementato (M1/M4) |
| `src/config.rs` | Config TOML validata | default, validazione, limiti (gap, enum, regex) | Pianificato (M4) |
| `src/errors.rs` | Errori tipizzati | `KtileError`, conversione exit code (2,4,5,6) | ✅ Completato (M1) |
| `src/daemon.rs` | Event loop e shutdown | polling eventi, coalescing, stop su segnali ≤1 s | Pianificato (M6) |
| `src/x11/connection.rs` | Connessione X11 | `connect`, `EwmhAtoms::new` | ✅ Completato (M1) |
| `src/x11/events.rs` | Selezione e traduzione eventi | `DomainEvent`, `select_root_events` | ✅ Implementato (M1) |
| `src/x11/ewmh.rs` | Lettura proprietà EWMH | `verify_wm`, `client_list` | ✅ Completato (M1) |
| `src/x11/icccm.rs` | Lettura metadati ICCCM | `WM_CLASS`, `WM_TRANSIENT_FOR`, normal hint (read-only) | Pianificato |
| `src/x11/monitor.rs` | Monitor e aree | `read_workspace_area` (RandR + strut/workarea) | ✅ Completato (M4) |
| `src/x11/applier.rs` | Applicazione geometrie | gate richieste, geometry-in-flight, BadWindow | Pianificato (M5) |
| `src/state/registry.rs` | Stato finestre | `WindowId`, `WindowState` | ✅ Implementato (M1) |
| `src/state/eligibility.rs` | Policy di esclusione | `is_eligible`, `ExcludeConfig` | ✅ Completato (M4) |
| `src/state/workspace.rs` | Stato per workspace | `WorkspaceId=(desktop,monitor)`, alberi layout | Pianificato |
| `src/state/reconciliation.rs` | Diff stato↔X11 | ricalcolo selettivo, idempotenza | Pianificato |
| `src/layout/geometry.rs` | Rettangoli e gap | `Rect`, `Gaps`, `compute_dwindle`, `compute_master_stack` | ✅ Completato (M2) |
| `src/layout/tree.rs` | Albero binario | insert/remove/promote (dwindle) | Pianificato (M3) |
| `src/layout/dwindle.rs` | Policy dwindle | split_mode/target/ratio, fallback minimi | Pianificato (M3) |
| `src/layout/master_stack.rs` | Policy master-stack | `new_status`, promozione, `mfact` | Pianificato (M3) |
| `src/layout/plan.rs` | Piano geometrico | `Plan`, `WindowMove`, `diff`, idempotenza | ✅ Completato (M2) |
| `docs/` | Documentazione | fonte di verità progettuale | In continuo aggiornamento |

## 4. Flusso dei dati e architettura logica

1. **Bootstrap:** `cli` → `config` → connessione X11 (`x11/connection`) → sanity check EWMH (`_NET_SUPPORTING_WM_CHECK`, exit 6 se essenziale assente — D6) → snapshot client/workspace/monitor (`x11/ewmh`, `x11/monitor`) → stato iniziale (`state/`).
2. **Ciclo di vita:** eventi X11 (`x11/events`) → `DomainEvent` → aggiornamento stato (`state/`) → **solo se lo stato cambia in modo rilevante** → piano (`layout/`, motore puro) → diff con stato atteso → richieste controllate (`x11/applier`).
3. **Sicurezza del flusso:** i ConfigureNotify propri vengono consumati dal filtro anti-feedback; `BadWindow` diventa `WindowGone`; piano identico allo stato atteso → zero richieste (idempotenza); max 3 ricalcoli per burst (anti-livelock).
4. **Confini invarianti:** `layout/` non conosce X11; `state/` non parla col server; `applier` è l'unico scrittore di geometrie; hint ICCCM mai riscritti (su Muffin/Mutter mai `WM_NORMAL_HINTS`, vincolo da analisi Cortile).
