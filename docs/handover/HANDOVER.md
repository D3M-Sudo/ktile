# HANDOVER — Ktile (2026-10-22)

> File di passaggio di consegne per una nuova sessione/agente.
> Branch di lavoro: `feature-complete-roadmap` (sviluppo M5 e aggiornamento documentazione). La CI dell'ultimo aggiornamento è stata confermata verde dall'utente il 2026-10-22.
> Fase: **M0–M4 implementati; M5 implementata con gate target ancora aperto**. Non creare PR né fare merge senza richiesta esplicita.

## 1. Cos'è Ktile (30 secondi)

Tiler X11 **esterno e cooperante** con Muffin/Cinnamon (Mint, sessione X11). Non è un compositor, non sostituisce il WM.
Riferimento UX Hyprland (dwindle a split binari, master-stack, gaps), replicato in modo deterministico senza effetti grafici.
Stack: Rust stable (1.94.0 in cloud) · backend unico **`x11rb`** · TOML (`serde`+`toml`) · CLI (`clap`) · errori (`thiserror`) · `cargo test`.
Regole permanenti: fonte di verità = `docs/ARCHITECTURE.md` + `docs/ROADMAP.md` con **aggiornamento contestuale obbligatorio** (ADR-13);
Muffin resta autorevole su focus/stacking/decorazioni/workspace.

## 2. Stato lavori (verità attuale)

| Area | Stato | Dettagli |
|---|---|---|
| Git | ramo corrente `feature-complete-roadmap` | Lavoro su ramo dedicato; verificare sempre HEAD/CI live prima di altri interventi. Nessuna PR o merge autorizzati automaticamente. |
| M0 baseline | ✅ completata e committata (`2c5c9f2`) | `docs/milestones/environment-baseline.md`: Mint 22.3, Cinnamon 6.6.9, Muffin 6.6.3, X11 `:0`, x86_64, tool X11 presenti, 4 desktop, workarea 1920×1040. |
| M1 Gate | ✅ **Chiuso in cloud** (2026-10-19) **e confermato su target** (2026-10-20) | Cloud: toolchain Rust 1.94.0 (ADR-15), `cargo build/fmt/clippy/test` verdi, integrazione Xvfb + openbox. Target: osservazioni residue B5/B6 eseguite (M1-findings §2c), grep 0 match rieseguito; conteggi test: 15 suite plain / 20 totali con `--ignored` (9+4+2+5). |
| M2 Core Geometrico | ✅ **Completato in cloud** (2026-10-19) | `src/layout/{geometry.rs, plan.rs}` e `tests/geometry.rs`. 25/25 fixture da `fixtures-geometry.json` verificate con esito positivo e invarianti rispettate. |
| Moduli Lettura | ✅ **Completati in cloud** (2026-10-19) | `src/x11/monitor.rs` (RandR + workarea -> `WorkspaceArea`, guardrail single-monitor) e `src/state/eligibility.rs` (`is_eligible` con 9 test unitari). |
| M4 Config/CLI/Reconcile | ✅ implementata | Config TOML (`src/config.rs`), CLI (`src/cli.rs`), reconciliation read-only count-based, lettore ICCCM e modelli workspace. La verifica target del calcolo area/pannelli resta da eseguire. |
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
Ktile/  (branch feature-complete-roadmap)
├── README.md, .gitignore, Cargo.toml, Cargo.lock
├── docs/
│   ├── ARCHITECTURE.md, ROADMAP.md      ← aggiornati contestualmente (ADR-13)
│   ├── handover/HANDOVER.md             ← questo file
│   ├── spec/        01, 02, 04, config.example.toml, cli-matrix.md
│   ├── research/    03 (EWMH/Muffin), 05 (roadmap M0–M8)
│   └── milestones/  environment-baseline.md, M1-findings.md, M1-residuals-user.md, fixtures-geometry.json
├── src/   lib|main|cli|errors|config, x11/{connection,ewmh,events,monitor,icccm,applier}, state/{registry,eligibility,workspace,reconciliation}, layout/{geometry,plan,dwindle,master_stack,tree}
└── tests/ applier_integration.rs, exit_codes.rs, geometry.rs, eligibility.rs, layout_tree.rs, layout_master.rs, cli_tests.rs, config_validation.rs, x11_integration.rs
```

## 5. Prossimi passi consentiti

1. **M5 — test target:** seguire `docs/ROADMAP.md` in una sessione Cinnamon/X11 reale. La CI Xvfb/Openbox è verde, ma non sostituisce l'accettazione su Muffin. Registrare PASS/FAIL e versione Cinnamon/Muffin.
2. **M5 — test race deterministico:** verificare se il test della distruzione fra controllo live e risposta WM può essere reso deterministico; non dichiararlo coperto solo perché è gestito l'errore `BadWindow`.
3. **M4 residuo:** testare in locale l'area utile con pannello laterale/auto-hide; la baseline osservata non mostrava `_NET_WM_STRUT(_PARTIAL)`, quindi ADR-12 resta da rivalutare.
4. **Documentazione:** risolvere la divergenza del percorso config (`~/.config/ktile/config.toml` nella specifica vs `~/.config/ktile/ktile.toml` e `./ktile.toml` nel codice) con decisione esplicita prima di modificare il contratto.
5. Non creare PR o fare merge senza richiesta esplicita dell'utente.

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
