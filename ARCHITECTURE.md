# ARCHITECTURE — Ktile

> Fonte di verità architetturale del progetto. **Regola operativa permanente:** ogni decisione architetturale, nuova funzione, cambio di stato di un modulo o riorganizzazione dei file aggiorna **contestualmente** questo file e/o `ROADMAP.md`. Ogni proposta di codice include l'aggiornamento corrispondente.
> Stato redazione: 2026-10-17 — progetto in fase **M0** (baseline da raccogliere sulla macchina target; nessun codice ancora creato).

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

**Stack:** Rust (stable recente) · backend X11 **`x11rb`** (unico, decisione presa in analisi) · config TOML (`serde`+`toml`) · CLI (`clap`) · errori (`thiserror`) · test nativi `cargo test` (integrazione X11 dietro feature `x11-tests`, marcata `#[ignore]`).

**Policy tool di sviluppo (ADR-14, 2026-10-19):** host Mint pulito, zero `sudo`.
- Tool **Python** mancanti → `uvx <tool>` per one-shot, oppure `.venv/` in repo (`uv venv .venv && uv pip install …`) per uso in sessione. `.venv/` è ignorato da git e mai committato; `uv.lock` solo se il tool serve stabilmente al progetto.
- Tool **X11/di sistema** mancanti (`xprop`, `xwininfo`, `xev`, `xrandr`, …) → **non installare**: registrare `assente` in baseline e chiedere all'utente. `venv`/`uvx` non coprono binari nativi.

## 2. Albero delle directory

Stato attuale (2026-10-17) — progetto documentale, codice non ancora inizializzato:

```text
<repo>/
├── ARCHITECTURE.md            ← questo file
├── ROADMAP.md                 ← milestone M0–M8, checklist, ADR
├── README.md                  ← documentazione progettuale (indice)
├── docs/
│   ├── 01-project-overview.md
│   ├── 02-architecture-rust-x11rb.md
│   ├── 03-ewmh-muffin-compatibility.md
│   ├── 04-functional-specs-dwindle-master.md
│   ├── 05-roadmap-m0-m8.md
│   ├── environment-baseline.md    (M0 — da creare)
│   └── M1-findings.md             (M1 — da creare)
└── src/  tests/  Cargo.toml  ...  (M1 in poi — non ancora presenti)
```

Struttura prevista del codice (dal doc 02, da realizzare milestone per milestone):

```text
src/
  main.rs  cli.rs  config.rs  errors.rs  daemon.rs
  x11/{mod.rs,connection.rs,events.rs,ewmh.rs,icccm.rs,monitor.rs,applier.rs}
  state/{mod.rs,registry.rs,eligibility.rs,workspace.rs,reconciliation.rs}
  layout/{mod.rs,geometry.rs,plan.rs,tree.rs,dwindle.rs,master_stack.rs}
tests/
  layout_tree.rs  layout_master.rs  geometry_edge.rs  eligibility.rs  fixtures/
```

## 3. Tabella dei moduli

| Path/File | Responsabilità | Funzioni principali | Stato |
|---|---|---|---|
| `src/main.rs` | Entry point, wiring, codici d'uscita | `main`, mappatura errori→exit | Pianificato |
| `src/cli.rs` | Argomenti CLI | parsing, `--check-config/--list-windows/--dry-run/--layout/--daemon` | Pianificato |
| `src/config.rs` | Config TOML validata | default, validazione, limiti (gap, enum, regex) | Pianificato |
| `src/errors.rs` | Errori tipizzati | `KtileError`, conversione exit code | Pianificato |
| `src/daemon.rs` | Event loop e shutdown | polling eventi, coalescing, stop su segnali ≤1 s | Pianificato |
| `src/x11/connection.rs` | Connessione X11 | connect, intern atomi, retry, check estensioni | Pianificato |
| `src/x11/events.rs` | Selezione e traduzione eventi | maschere root/client, `DomainEvent` | Pianificato |
| `src/x11/ewmh.rs` | Lettura proprietà EWMH | `_NET_*` (tabella doc 03), fallback | Pianificato |
| `src/x11/icccm.rs` | Lettura metadati ICCCM | `WM_CLASS`, `WM_TRANSIENT_FOR`, normal hint (read-only) | Pianificato |
| `src/x11/monitor.rs` | Monitor e aree | RandR heads, strut → `WorkspaceArea` | Pianificato |
| `src/x11/applier.rs` | Applicazione geometrie | gate richieste, geometry-in-flight, BadWindow | Pianificato |
| `src/state/registry.rs` | Stato finestre | `Active/Suspended/Excluded/Gone`, posizioni | Pianificato |
| `src/state/eligibility.rs` | Policy di esclusione | tipi/stati/transient, regex config | Pianificato |
| `src/state/workspace.rs` | Stato per workspace | `WorkspaceId=(desktop,monitor)`, alberi layout | Pianificato |
| `src/state/reconciliation.rs` | Diff stato↔X11 | ricalcolo selettivo, idempotenza | Pianificato |
| `src/layout/geometry.rs` | Rettangoli e gap | `Rect`, split, clamp, invarianti | Pianificato |
| `src/layout/tree.rs` | Albero binario | insert/remove/promote (dwindle) | Pianificato |
| `src/layout/dwindle.rs` | Policy dwindle | split_mode/target/ratio, fallback minimi | Pianificato |
| `src/layout/master_stack.rs` | Policy master-stack | `new_status`, promozione, `mfact` | Pianificato |
| `src/layout/plan.rs` | Piano geometrico | `Plan{moves}`, diff, idempotenza | Pianificato |
| `docs/` | Documentazione | fonte di verità progettuale | In sviluppo |

## 4. Flusso dei dati e architettura logica

1. **Bootstrap:** `cli` → `config` → connessione X11 (`x11/connection`) → sanity check EWMH (`_NET_SUPPORTING_WM_CHECK`, exit 6 se essenziale assente — D6) → snapshot client/workspace/monitor (`x11/ewmh`, `x11/monitor`) → stato iniziale (`state/`).
2. **Ciclo di vita:** eventi X11 (`x11/events`) → `DomainEvent` → aggiornamento stato (`state/`) → **solo se lo stato cambia in modo rilevante** → piano (`layout/`, motore puro) → diff con stato atteso → richieste controllate (`x11/applier`).
3. **Sicurezza del flusso:** i ConfigureNotify propri vengono consumati dal filtro anti-feedback; `BadWindow` diventa `WindowGone`; piano identico allo stato atteso → zero richieste (idempotenza); max 3 ricalcoli per burst (anti-livelock).
4. **Confini invarianti:** `layout/` non conosce X11; `state/` non parla col server; `applier` è l'unico scrittore di geometrie; hint ICCCM mai riscritti (su Muffin/Mutter mai `WM_NORMAL_HINTS`, vincolo da analisi Cortile).

Stati e decisioni architetturali: registro ADR in `ROADMAP.md`, sezione "Registro decisioni (ADR)".
