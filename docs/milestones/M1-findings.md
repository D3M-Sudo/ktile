# M1-findings — Ktile M1-prep (PARZIALE, 2026-10-19)

> M1 completa richiede la toolchain Rust (assente sull'host, verificato M0).
> Quanto segue è raccolto con soli tool di sistema (`xprop`, `xwininfo`, `xrandr`) + prototipo Python stdlib usa-e-getta (`/tmp/ktile_proto.py`, NON committato).
> Codice Rust in repo = **DRAFT non compilato, non testato**.

## 1. Codice DRAFT (non verificato)

| File | Stato |
|---|---|
| `Cargo.toml` (`x11rb` + `thiserror`) | DRAFT — dipendenze minime M1, lock non generabile senza cargo |
| `src/main.rs`, `src/cli.rs`, `src/errors.rs` | DRAFT — wiring + exit codes 2/4/5/6 |
| `src/x11/{mod,connection,ewmh,events}.rs` | DRAFT — firme + logica documentata dai dati M0 |
| `src/state/{mod,registry}.rs` | DRAFT — modello Active/Suspended/Excluded/Gone |
| `tests/exit_codes.rs` | DRAFT — 3 test da eseguire con toolchain |

## 2. Verifiche EWMH chiuse in M1-prep (verificato)

| Ipotesi doc 03 §9 | Esito |
|---|---|
| 8. Nome WM | ✅ **chiusa**: `Mutter (Muffin)` su `0x2600006` (doppia verifica root→WM) |
| 7. Workspace/viewport | ✅ parziale: 4 desktop (`Spazio di lavoro 1..4`), `_NET_DESKTOP_VIEWPORT = 0,0` |
| Strut pannello (M0 §6) | ✅ **chiusa con fallback documentato**: nessuna finestra root-child espone `_NET_WM_STRUT_PARTIAL` (pannello Cinnamon fuori da `_NET_CLIENT_LIST`, geometrie 10×10/1×1); area utile da `_NET_WORKAREA` = 1920×1040 (40px riservati in basso) — conforme ADR-12 (strut primario, WORKAREA controllo incrociato; qui WORKAREA è l'unica fonte osservabile) |
| 4. `_GTK_FRAME_EXTENTS` CSD | ✅ parziale: Firefox CSD → `0,0,0,0`; devin-desktop → proprietà assente; `_NET_FRAME_EXTENTS` assente su entrambe (maximized) |
| Dialoghi/transient | ⚠️ nessun dialogo aperto durante la sessione: `WM_TRANSIENT_FOR` assente su tutti i campioni NORMAL — resta **ipotesi** |
| 5/6/9/10/1/2/3 | **ipotesi** (richiedono osservazione 30s via `ktile` o sessione interattiva) |

## 3. Geometria validata via prototipo (verificato, N=0..8)

Prototipo `/tmp/ktile_proto.py` (stdlib, usa-e-getta): `Rect/split` + `dwindle` (longest-side/alternating) + `master-stack` (mfact 0.55).
Invarianti su N=0..8: dentro area ✅, lati ≥1px ✅, nessuna sovrapposizione ✅, determinismo D3 ✅.
Casi documentati: `single → (8,8,1904,1024)`; `two-dwindle → (8,8,948,1024)+(964,8,948,1024)`; `two-master → (8,8,1042,1024)+(1058,8,854,1024)` (riflessi in `fixtures-geometry.json`).
**Limite noto**: N grandi con foglie < gap richiedono policy clamp+log (spec M2) — il prototipo ingenuo falliva oltre N≈20, caso registrato non nascosto.

## 4. Gate M1

- [x] Zero API di scrittura geometrica nel DRAFT (solo lettura: `get_property`, maschere eventi)
- [ ] `cargo fmt --check` / `clippy` / `test` — **sospesi (no toolchain)**
- [ ] Osservazione 30s via `ktile` — **sospesa** (manuale `xev` possibile ma non equivalente)
- [ ] Manuali Cinnamon reale + Ctrl-C pulito + avvio senza DISPLAY — **sospesi**
