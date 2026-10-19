# HANDOVER — Ktile (2026-10-19)

> File di passaggio di consegne per una nuova sessione/agente.
> Branch: `feature` @ `325e15a` · working tree pulito · tutto locale (push/PR solo su richiesta utente).

## 1. Cos'è Ktile (30 secondi)

Tiler X11 **esterno e cooperante** con Muffin/Cinnamon (Mint, sessione X11). Non è un compositor, non sostituisce il WM.
Riferimento UX Hyprland (dwindle a split binari, master-stack, gaps), replicato in modo deterministico senza effetti grafici.
Stack: Rust stable · backend unico **`x11rb`** · TOML (`serde`+`toml`) · CLI (`clap`) · errori (`thiserror`) · `cargo test`.
Regole permanenti: fonte di verità = `docs/ARCHITECTURE.md` + `docs/ROADMAP.md` con **aggiornamento contestuale obbligatorio** (ADR-13);
host pulito, zero `sudo` (ADR-14); Muffin resta autorevole su focus/stacking/decorazioni/workspace.

## 2. Stato lavori (verità attuale, non promesse)

| Area | Stato | Dettagli |
|---|---|---|
| Git | ✅ `feature` @ `325e15a`, clean, solo locale | `main/development/testing/feature` esistono su `origin` @ `1f6233c`; `feature` locale avanti di 3 commit non pushati. Ruleset `ktile promotion flow`: `main/development/testing` solo via PR, `feature` libero. |
| M0 baseline | ✅ completata e committata (`2c5c9f2`) | `docs/milestones/environment-baseline.md`: Mint 22.3, Cinnamon 6.6.9, Muffin 6.6.3, X11 `:0`, x86_64, tool X11 presenti, matrice EWMH, 4 desktop, workarea 1920×1040. |
| M1-prep | ⚠️ parziale e committato (`325e15a`) | Codice Rust = **DRAFT non compilato, non testato** (ogni file marcato). Verifiche manuali EWMH chiuse (WM `Mutter (Muffin)`, client list, strut→fallback WORKAREA). Gate cargo sospeso. Vedi `docs/milestones/M1-findings.md`. |
| Docs | ✅ riorganizzate (`6e3fcf2`) | `docs/{ARCHITECTURE,ROADMAP}.md`, `spec/` (01,02,04 + `config.example.toml`, `cli-matrix.md`), `research/` (03,05), `milestones/` (baseline, M1-findings, `fixtures-geometry.json`). Root: solo `README.md` + `.gitignore` + `Cargo.toml` + `src/` + `tests/`. |
| Geometria | ✅ validata via prototipo usa-e-getta | `/tmp/ktile_proto.py` (stdlib, NON in repo): invarianti N=0..8, casi in `fixtures-geometry.json`. Limite noto: N grandi → policy clamp+log (M2). |
| Toolchain Rust | ❌ **assente, stop M1** | `rustc/cargo/rustup` non trovati, `~/.cargo` e `~/.rustup` inesistenti (M0 §4). **Utente: zero installazioni** — non proporre `rustup/apt`; si lavora no-toolchain (Step 5). |

## 3. Decisioni chiuse (non rinegoziare senza ADR)

D1 nome `Ktile`/binario `ktile` · D2 gap esterno chiave unica `outer` · D3 ordine `stable`, `split_target=active` ·
D4 zero eleggibili → exit 0 · D5 core dwindle+master-stack · D6 exit 6 solo EWMH essenziale assente, no whitelist WM ·
D7 `new_status=slave` · D8 sospese → slot originario o nuova foglia · D9 sticky escluse v1 (rivalutare M7) ·
ADR-10 backend unico `x11rb` · ADR-11 mai riscrivere `WM_NORMAL_HINTS` · ADR-12 area da strut+RandR, WORKAREA solo cross-check ·
ADR-13 fonte di verità contestuale · ADR-14 tool Python→`uvx`/`.venv`, tool X11 assenti→registrare e chiedere, mai `sudo`/`apt`.

## 4. Struttura repo (dove sta cosa)

```
Ktile/  (branch feature)
├── README.md, .gitignore, Cargo.toml (DRAFT)
├── docs/
│   ├── ARCHITECTURE.md, ROADMAP.md      ← aggiornare contestualmente a ogni cambio
│   ├── handover/HANDOVER.md             ← questo file
│   ├── spec/        01, 02, 04, config.example.toml, cli-matrix.md
│   ├── research/    03 (EWMH/Muffin), 05 (roadmap M0–M8 + template vibe-coding)
│   └── milestones/  environment-baseline.md, M1-findings.md, fixtures-geometry.json
├── src/   main|cli|errors (DRAFT), x11/{mod,connection,ewmh,events} (DRAFT), state/{mod,registry} (DRAFT)
└── tests/ exit_codes.rs (DRAFT)
```

## 5. Prossimi passi consentiti (no-toolchain)

1. **Chiudere i residui EWMH** con soli `xprop`/`xrandr`: dialogo/transient reale (`WM_TRANSIENT_FOR`), `_NET_WM_STATE` fullscreen/HIDDEN, pannello laterale/auto-hide se configurabile, `_NET_FRAME_EXTENTS` su finestra SSD non-maximized.
2. **Arricchire le fixture** (`fixtures-geometry.json`): N=3..8 attesi, gap estremi, aree piccole, coordinate negative — restano i casi dei futuri `cargo test` M2.
3. **Estendere il Rust DRAFT** (sempre marcato non compilato): `monitor.rs` (lettura), `eligibility.rs` (matrice doc 04 §3), `geometry.rs`/`plan.rs` puri (port del prototipo `/tmp/ktile_proto.py`).
4. **Mai senza toolchain**: `cargo fmt/clippy/test/build`, `cargo add`, generazione `Cargo.lock`, osservazione 30s via `ktile`.
5. **Mai senza autorizzazione esplicita**: commit/push/PR/merge, cambio branch, `sudo`/`apt`, installazioni (rustup incluso), spostare finestre, modificare file fuori scope milestone.

## 6. Comandi di verifica rapida (read-only)

```bash
git status --porcelain; git branch --show-current; git log --oneline -4
printenv XDG_SESSION_TYPE DISPLAY XDG_CURRENT_DESKTOP
xprop -root _NET_SUPPORTING_WM_CHECK _NET_CLIENT_LIST _NET_ACTIVE_WINDOW _NET_WORKAREA
grep -rniE 'configure_window|moveresize|set_geometry' src/ tests/ || echo "OK: nessuna scrittura geometrica"
```

## 7. Rischi e punti aperti

- Rust assente → M1 non chiudibile; il DRAFT potrebbe contenere errori di compilazione (da verificare con toolchain futura).
- Strut pannello non osservato direttamente → area da `_NET_WORKAREA` come fallback documentato (ADR-12 rispettato, da re-verificare con pannello spostato).
- Nessun dialogo/transient campionato → policy esclusione non verificata sul campo (resta ipotesi doc 03 §9).
- `origin/feature` fermo a `6e3fcf2`: i commit `2c5c9f2` (M0) e `325e15a` (M1-prep) sono solo locali — push/PR verso `testing` solo su richiesta utente (ruleset: PR obbligatoria).
