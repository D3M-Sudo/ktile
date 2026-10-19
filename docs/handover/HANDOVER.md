# HANDOVER — Ktile (2026-10-19)

> File di passaggio di consegne per una nuova sessione/agente.
> Branch: `feature` @ `aca2c28` (stato iniziale task) · working tree pulito · pushato su `origin/feature`.

## 1. Cos'è Ktile (30 secondi)

Tiler X11 **esterno e cooperante** con Muffin/Cinnamon (Mint, sessione X11). Non è un compositor, non sostituisce il WM.
Riferimento UX Hyprland (dwindle a split binari, master-stack, gaps), replicato in modo deterministico senza effetti grafici.
Stack: Rust stable (1.94.0 in cloud) · backend unico **`x11rb`** · TOML (`serde`+`toml`) · CLI (`clap`) · errori (`thiserror`) · `cargo test`.
Regole permanenti: fonte di verità = `docs/ARCHITECTURE.md` + `docs/ROADMAP.md` con **aggiornamento contestuale obbligatorio** (ADR-13);
Muffin resta autorevole su focus/stacking/decorazioni/workspace.

## 2. Stato lavori (verità attuale)

| Area | Stato | Dettagli |
|---|---|---|
| Git | ✅ `feature` @ `aca2c28`, clean | Working tree pulito, commit convenzionali pronti per push su `origin/feature`. Ruleset: `main/development/testing` solo via PR. |
| M0 baseline | ✅ completata e committata (`2c5c9f2`) | `docs/milestones/environment-baseline.md`: Mint 22.3, Cinnamon 6.6.9, Muffin 6.6.3, X11 `:0`, x86_64, tool X11 presenti, 4 desktop, workarea 1920×1040. |
| M1 Gate (Cloud) | ✅ **Chiuso in cloud** (2026-10-19) | Toolchain Rust 1.94.0 installata in sandbox cloud (ADR-15). `cargo build/fmt/clippy/test` verdi. Test d'integrazione X11 eseguiti sotto Xvfb + openbox (marcati "verificato su WM di test, da confermare su Muffin"). |
| M2 Core Geometrico | ✅ **Completato in cloud** (2026-10-19) | `src/layout/{geometry.rs, plan.rs}` e `tests/geometry.rs`. 25/25 fixture da `fixtures-geometry.json` verificate con esito positivo e invarianti rispettate. |
| Moduli Lettura | ✅ **Completati in cloud** (2026-10-19) | `src/x11/monitor.rs` (RandR + workarea -> `WorkspaceArea`, guardrail single-monitor) e `src/state/eligibility.rs` (`is_eligible` con 9 test unitari). |
| Residui B5/B6 Target | 📋 **Trasferiti all'utente** | Protocollo pronto-all'uso consegnato in `docs/milestones/M1-residuals-user.md` per esecuzione in 10 min sulla macchina fisica dell'utente. |

## 3. Decisioni chiuse (non rinegoziare senza ADR)

D1 nome `Ktile`/binario `ktile` · D2 gap esterno chiave unica `outer` · D3 ordine `stable`, `split_target=active` ·
D4 zero eleggibili → exit 0 · D5 core dwindle+master-stack · D6 exit 6 solo EWMH essenziale assente, no whitelist WM ·
D7 `new_status=slave` · D8 sospese → slot originario o nuova foglia · D9 sticky escluse v1 (rivalutare M7) ·
ADR-10 backend unico `x11rb` · ADR-11 mai riscrivere `WM_NORMAL_HINTS` · ADR-12 area da strut+RandR, WORKAREA solo cross-check ·
ADR-13 fonte di verità contestuale · ADR-14 isolamento python/venv · ADR-15 toolchain Rust in ambiente cloud sandbox.

## 4. Struttura repo (dove sta cosa)

```text
Ktile/  (branch feature)
├── README.md, .gitignore, Cargo.toml, Cargo.lock
├── docs/
│   ├── ARCHITECTURE.md, ROADMAP.md      ← aggiornati contestualmente (ADR-13)
│   ├── handover/HANDOVER.md             ← questo file
│   ├── spec/        01, 02, 04, config.example.toml, cli-matrix.md
│   ├── research/    03 (EWMH/Muffin), 05 (roadmap M0–M8)
│   └── milestones/  environment-baseline.md, M1-findings.md, M1-residuals-user.md, fixtures-geometry.json
├── src/   lib|main|cli|errors, x11/{connection,ewmh,events,monitor}, state/{registry,eligibility}, layout/{geometry,plan}
└── tests/ exit_codes.rs, geometry.rs, eligibility.rs, x11_integration.rs
```

## 5. Prossimi passi consentiti

1. **M3: Dwindle split_target attivo + albero binario completo** (`layout/tree.rs`, `layout/dwindle.rs`, `layout/master_stack.rs` con test di inserimento/rimozione/promozione).
2. **M4: Configurazione TOML e dry-run** (`config.rs`, CLI `--check-config`, `--dry-run`).

## 6. Comandi di verifica rapida

```bash
cargo check
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test x11_integration -- --ignored
grep -rniE 'configure_window|moveresize' src/ tests/ || echo "OK: zero match"
```
