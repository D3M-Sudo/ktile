# HANDOVER — Ktile (2026-10-20)

> File di passaggio di consegne per una nuova sessione/agente.
> Branch: `feature` @ `661305e` · working tree pulito · pushato su `origin/feature`.
> Fase: **M0–M3 completati e chiusi (2026-10-20)** · Prossima: **M5**.

## 1. Cos'è Ktile (30 secondi)

Tiler X11 **esterno e cooperante** con Muffin/Cinnamon (Mint, sessione X11). Non è un compositor, non sostituisce il WM.
Riferimento UX Hyprland (dwindle a split binari, master-stack, gaps), replicato in modo deterministico senza effetti grafici.
Stack: Rust stable (1.94.0 in cloud) · backend unico **`x11rb`** · TOML (`serde`+`toml`) · CLI (`clap`) · errori (`thiserror`) · `cargo test`.
Regole permanenti: fonte di verità = `docs/ARCHITECTURE.md` + `docs/ROADMAP.md` con **aggiornamento contestuale obbligatorio** (ADR-13);
Muffin resta autorevole su focus/stacking/decorazioni/workspace.

## 2. Stato lavori (verità attuale)

| Area | Stato | Dettagli |
|---|---|---|
| Git | ✅ `feature` @ `661305e`, clean | Working tree pulito. Ruleset: `main/development/testing` solo via PR. Merge commit (come PR #1). |
| M0 baseline | ✅ completata e committata (`2c5c9f2`) | `docs/milestones/environment-baseline.md`: Mint 22.3, Cinnamon 6.6.9, Muffin 6.6.3, X11 `:0`, x86_64, tool X11 presenti, 4 desktop, workarea 1920×1040. |
| M1 Gate | ✅ **Chiuso in cloud** (2026-10-19) **e confermato su target** (2026-10-20) | Cloud: toolchain Rust 1.94.0 (ADR-15), `cargo build/fmt/clippy/test` verdi, integrazione Xvfb + openbox. Target: osservazioni residue B5/B6 eseguite (M1-findings §2c), grep 0 match rieseguito; conteggi test: 15 suite plain / 20 totali con `--ignored` (9+4+2+5). |
| M2 Core Geometrico | ✅ **Completato in cloud** (2026-10-19) | `src/layout/{geometry.rs, plan.rs}` e `tests/geometry.rs`. 25/25 fixture da `fixtures-geometry.json` verificate con esito positivo e invarianti rispettate. |
| Moduli Lettura | ✅ **Completati in cloud** (2026-10-19) | `src/x11/monitor.rs` (RandR + workarea -> `WorkspaceArea`, guardrail single-monitor) e `src/state/eligibility.rs` (`is_eligible` con 9 test unitari). |
| M4 Config/CLI/Reconcile | ✅ **Merged in `feature`** (2026-10-20) | PR #2 merged, merge commit `661305e` (branch `feature-12655789238683274081`, eliminata). Deliverable: config TOML (`src/config.rs`), CLI `clap` (`src/cli.rs`), `reconcile_read_only` count-based (`src/state/reconciliation.rs`), lettore ICCCM (`src/x11/icccm.rs`), workspace (`src/state/workspace.rs`). CI GitHub Actions ripristinata (ADR-16). Gate: `cargo test` 48 verdi, `x11_integration` 5/5 sotto Xvfb+openbox. |
| Residui B5/B6 Target | ✅ **Completati su target** (2026-10-20) | Eseguiti con il protocollo `docs/milestones/M1-residuals-user.md` su Mint 22.3 / Cinnamon 6.6.9 / Muffin 6.6.3 (X11 `:0`). B5: HIDDEN in `_NET_WM_STATE`, nessun Unmap/Map al minimize, permanenza in `_NET_CLIENT_LIST` (verificato). B6: pannello laterale + revert = baseline (verificato); nessun strut esposto → ADR-12 da re-verificare. Output grezzi: M1-findings §2c. |

## 3. Decisioni chiuse (non rinegoziare senza ADR)

D1 nome `Ktile`/binario `ktile` · D2 gap esterno chiave unica `outer` · D3 ordine `stable`, `split_target=active` ·
D4 zero eleggibili → exit 0 · D5 core dwindle+master-stack · D6 exit 6 solo EWMH essenziale assente, no whitelist WM ·
D7 `new_status=slave` · D8 sospese → slot originario o nuova foglia · D9 sticky escluse v1 (rivalutare M7) ·
ADR-10 backend unico `x11rb` · ADR-11 mai riscrivere `WM_NORMAL_HINTS` · ADR-12 area da strut+RandR, WORKAREA solo cross-check ·
ADR-13 fonte di verità contestuale · ADR-14 isolamento python/venv · ADR-15 toolchain Rust in ambiente cloud sandbox ·
ADR-16 build/test Rust prima scelta in CI (fallback `cargo` su host solo se toolchain già autorizzata, mai installazioni di sistema).

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
├── src/   lib|main|cli|errors|config, x11/{connection,ewmh,events,monitor,icccm}, state/{registry,eligibility,workspace,reconciliation}, layout/{geometry,plan,dwindle,master_stack,tree}
└── tests/ exit_codes.rs, geometry.rs, eligibility.rs, layout_tree.rs, layout_master.rs, cli_tests.rs, config_validation.rs, x11_integration.rs
```

## 5. Prossimi passi consentiti

> Stato 2026-10-20: M1 (B5/B6) **chiusi** e **M4 merged** (PR #2 → `661305e`) — nessun residuo M1 né M4 aperto; si prosegue da M5.

1. **M5: prossimo milestone** — definire l'ambito in `docs/ROADMAP.md` prima di implementare (ADR-13).
2. **Verificare il gate read-only sul target** dopo il merge: `grep -rniE 'configure_window|moveresize' src/ tests/` → 0 match.
3. **Promuovere `feature` verso `main`/`development`** solo via PR, se previsto dal ruleset del repository.

## 6. Comandi di verifica rapida

> **ADR-16: prima scelta la CI GitHub Actions** (`.github/workflows/ci.yml`).
> Build, fmt, clippy e test girano sui runner (job `check` + `test` con Xvfb+openbox):
> verificare con `gh run list` / `gh pr checks` oppure attendere i check sulla PR.
> Fallback `cargo` su host solo se toolchain user-level già installata/autorizzata
> (amendment ADR-15); mai proporre `sudo`/`apt install`.

```bash
gh run list --branch feature --limit 5
gh pr checks <n>
grep -rniE 'configure_window|moveresize' src/ tests/ || echo "OK: zero match"
```
