# M1-findings — Ktile M1-prep + M1-residuals + Cloud Gate M1/M2 (2026-10-19)

> **Stato M1 in Cloud**: Gate M1 sbloccato e chiuso in ambiente cloud con toolchain Rust 1.94.0 (ADR-15).
> Esiti build/clippy/test verdi in cloud; test d'integrazione eseguiti sotto Xvfb + openbox.
> **Importante**: Gli esiti X11 registrati in cloud sono contrassegnati come *"verificato su WM di test (openbox), da confermare su Muffin"*.
> Residui B5 e B6 trasferiti all'utente tramite il protocollo `docs/milestones/M1-residuals-user.md`.

## 1. Stato dei moduli in Repo (Verificato in Cloud)

| File / Modulo | Esito Build / Test | Note |
|---|---|---|
| `Cargo.toml` / `Cargo.lock` | ✅ Build OK (`x11rb`, `thiserror`, `serde`, `serde_json`) | Dipendenze minime tracciate |
| `src/main.rs`, `src/cli.rs`, `src/errors.rs` | ✅ Build & Test OK (`tests/exit_codes.rs`) | Codici d'uscita 2, 4, 5, 6 verificati |
| `src/x11/{connection,ewmh,events,monitor}.rs` | ✅ Build & Test OK (`tests/x11_integration.rs`) | Connessione, EWMH, RandR + workarea |
| `src/state/{registry,eligibility}.rs` | ✅ Build & Test OK (`tests/eligibility.rs`) | Policy pura di eleggibilità e 9 casi testati |
| `src/layout/{geometry,plan,mod}.rs` | ✅ Build & Test OK (`tests/geometry.rs`) | Motore puro, 25/25 fixture verdi, plan idempotente |

## 2. Esiti Test d'Integrazione X11 (Xvfb + openbox)

Esecuzione: `cargo test --test x11_integration -- --ignored --nocapture`

| Scenario | Esito | Note |
|---|---|---|
| Connessione senza `DISPLAY` | ✅ **Exit code 4** | Verificato in cloud — *da confermare su Muffin* |
| `Xvfb` senza WM EWMH | ✅ **Exit code 6** | Verificato su WM di test — *da confermare su Muffin* |
| `Xvfb` + `openbox` | ✅ **WM = 'Openbox'**, `_NET_CLIENT_LIST` letta | Verificato su WM di test — *da confermare su Muffin* |
| `BadWindow` su finestra morta | ✅ **Gestita senza panic** | Verificato su WM di test — *da confermare su Muffin* |
| `SIGINT` / Shutdown | ✅ **Shutdown cooperativo ≤1s** | Verificato in cloud — *da confermare su Muffin* |

## 3. Motore Geometrico M2 (25 Fixture)

Esecuzione: `cargo test --test geometry`
- **Fixture caricate**: 25/25 da `docs/milestones/fixtures-geometry.json`.
- **Invarianti**: dentro area di lavoro, dimensioni ≥ 1x1, nessuna sovrapposizione.
- **Convenzioni ratificate**:
  1. `outer` applicato una volta sull'area di lavoro; `inner` tra le due metà di ogni split.
  2. Dwindle: albero binario persistente, split `longest-side` default (`width >= height` -> orizzontale), nuova finestra in `second`, foglia target = MRU.
  3. Master-Stack: `mfact` applicato a `(usable_width - inner_gap)`, master floor, slave prende il resto. I pixel di resto nella divisione verticale degli slave sono distribuiti a 1px ciascuno per i primi `rem` slave.
  4. Rect: conversioni con `saturating_*`/`checked_*`, clamp aree piccole senza panic.
  5. Plan: diff tra stato corrente ed atteso. Se identici, il piano risulta vuoto (idempotenza).

## 4. Gate M1 Read-Only

- Controllo anti-scrittura geometrica:
  `grep -rniE 'configure_window|moveresize' src/ tests/` → **0 match (esattamente zero corrispondenze)**.
- `cargo fmt --check`: **passato (zero diff)**.
- `cargo clippy --all-targets -- -D warnings`: **0 avvisi / 0 errori**.
- `cargo test`: **15 test passati, 0 falliti**.

## 5. Appendice: Output e Prove Rilevate

### Command Cargo Check & Test Output
```text
running 9 tests in tests/eligibility.rs ... 9 passed
running 4 tests in tests/exit_codes.rs ... 4 passed
running 2 tests in tests/geometry.rs ... 2 passed
running 5 tests in tests/x11_integration.rs ... 5 passed (Xvfb + openbox)
```
