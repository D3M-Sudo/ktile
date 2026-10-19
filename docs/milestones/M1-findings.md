# M1-findings — Ktile M1-prep + M1-residuals (2026-10-19)

> M1 completa richiede la toolchain Rust (assente sull'host, verificato M0).
> Quanto segue è raccolto con soli tool di sistema (`xprop`, `xwininfo`, `xrandr`, `xev`) + prototipo Python stdlib usa-e-getta (`/tmp/ktile_proto.py`, NON committato).
> Codice Rust in repo = **DRAFT non compilato, non testato**.
> Sessione M1-residuals: `feature @ 444d79d`, X11 `:0`, Cinnamon 6.6.9 / Muffin 6.6.3 (finestra WM `0x2600006` = `Mutter (Muffin)`).
> Legenda: **verificato** = osservato con comando/output allegato · **ipotesi** = non osservato.

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
| 4. `_GTK_FRAME_EXTENTS` CSD | ✅ parziale: Firefox CSD → `0,0,0,0`; devin-desktop → proprietà assente; `_NET_FRAME_EXTENTS` assente su entrambe (maximized). **Esteso in §2b**: `_NET_FRAME_EXTENTS = 0,0,36,0` **verificato** su Nemo SSD non-maximized (B2) |
| Dialoghi/transient | ✅ **chiusa in §2b (B1)**: Zenity `--info` → `WINDOW_TYPE_DIALOG`, `WM_TRANSIENT_FOR` assente (verificato-assente); policy esclusione via type applicabile |
| Fullscreen `_NET_WM_STATE` | ✅ **chiusa in §2b (B3)**: `FULLSCREEN + FOCUSED` verificato su Firefox (F11); Nemo F11 → solo MAXIMIZED (evidenza negativa) |
| Minimize/HIDDEN + client list | ✅ **chiusa in §2b (B4)**: `FOCUSED → HIDDEN` al t=5, proprietà non eliminata, desktop 0, WID resta in `_NET_CLIENT_LIST` |
| Sequenza Unmap (xev) | ⚠️ **non eseguito — motivo tecnico (B5)**: `xev -id` muto su finestra già mappata (2 tentativi); B4/HIDDEN come surrogata per D8 |
| Pannello auto-hide/laterale | ⚠️ **non eseguito — utente ha declinato modifica config (B6)**; resta **ipotesi** (doc 03 §9.2/9.6) |
| 9/10/1/3 | **ipotesi** (9 alt-tab, 10 edge-tiling, 1 geometrie applicate, 3 messaggi client — richiedono `ktile` o sessione dedicata) |

## 2b. Verifiche EWMH chiuse in M1-residuals (2026-10-19)

Sessione: `feature @ 444d79d` · `XDG_SESSION_TYPE=x11`, `DISPLAY=:0`, `XDG_CURRENT_DESKTOP=X-Cinnamon` ·
root: `_NET_SUPPORTING_WM_CHECK=0x2600006`, stacking `0x2a00021, 0x5200017, 0x5400004`, active `0x5400004`,
`_NET_WORKAREA=0,0,1920,1040 ×4`, desktop `0/4` · 73 root-child, zero strut con valori >0 (conferma fallback WORKAREA).
Raccolte grezze fuori repo: `/tmp/ktile-ewmh-A.txt` (95 righe), `-B1/B2/B3/B4.txt`. Zero finestre spostate/ridimensionate, zero config modificate.

| # | Ipotesi doc 03 §9 / eleggibilità doc 04 §3 | Passo | Esito |
|---|---|---|---|
| — | Dialoghi/transient esclusi di default (doc 04 §3.4) | B1 | ✅ **verificato**: `zenity --info` → `WINDOW_TYPE=_NET_WM_WINDOW_TYPE_DIALOG`, `STATE=FOCUSED`, `CLASS="zenity","Zenity"`, `NAME="Informazioni"`; `WM_TRANSIENT_FOR` **assente** (verificato-assente su dialogo modale Zenity: Muffin classifica via type, transient non obbligatorio) |
| — | `_NET_FRAME_EXTENTS` su SSD non-maximized (doc 03 §5) | B2 | ✅ **verificato**: Nemo flottante non-maximized (`STATE=FOCUSED` solo) → `_NET_FRAME_EXTENTS=0,0,36,0` (titlebar SSD 36px top); `_GTK_FRAME_EXTENTS` assente (non-CSD, atteso) |
| — | Fullscreen segnalato via `_NET_WM_STATE` (doc 04 §3.3) | B3 | ✅ **verificato** (al retry): Firefox F11 → `STATE=FULLSCREEN,FOCUSED`, `TYPE=NORMAL`. Evidenza negativa: Nemo F11 → `MAXIMIZED_HORZ+VERT,FOCUSED` senza FULLSCREEN (F11 ≠ fullscreen su Nemo) |
| 5 | Sequenza minimize: HIDDEN / proprietà / client list (doc 04 §3.6, D8) | B4 | ✅ **verificato**: `WID=0x5000006`, t=1–4 `FOCUSED`, t=5–10 `HIDDEN` (proprietà non eliminata), `_NET_WM_DESKTOP=0` costante; `_NET_CLIENT_LIST=0x2a00021,0x5200017,0x5400004,0x5000006` → WID presente da minimizzata |
| 5 | Sequenza Unmap/MapNotify via xev | B5 | ⚠️ **non eseguito — motivo tecnico**: `timeout 15 xev -id $WID` muto in 2 tentativi (solo `WID=` nel file); selettore tardivo senza SubstructureNotify sul parent. Surrogata: B4/HIDDEN sufficiente per D8 |
| 2/6 | `_NET_WORKAREA`/strut con pannello laterale/auto-hide (ADR-12) | B6 | ⚠️ **non eseguito — utente ha declinato modifica configurazione pannello**; nessun ADR nuovo (ADR-12 invariato) |

## 3. Geometria validata via prototipo (verificato, N=0..8)

Prototipo `/tmp/ktile_proto.py` (stdlib, usa-e-getta): `Rect/split` + `dwindle` (longest-side/alternating) + `master-stack` (mfact 0.55).
Invarianti su N=0..8: dentro area ✅, lati ≥1px ✅, nessuna sovrapposizione ✅, determinismo D3 ✅.
Casi documentati: `single → (8,8,1904,1024)`; `two-dwindle → (8,8,948,1024)+(964,8,948,1024)`; `two-master → (8,8,1042,1024)+(1058,8,854,1024)` (riflessi in `fixtures-geometry.json`).
**Limite noto**: N grandi con foglie < gap richiedono policy clamp+log (spec M2) — il prototipo ingenuo falliva oltre N≈20, caso registrato non nascosto.

## 4. Gate M1

- [x] Zero API di scrittura geometrica nel DRAFT (solo lettura: `get_property`, maschere eventi)
- [ ] `cargo fmt --check` / `clippy` / `test` — **sospesi (no toolchain)**
- [ ] Osservazione 30s via `ktile` — **sospesa** (manuale `xev` possibile ma non equivalente)
- [x] Manuali Cinnamon reale B1–B4 (2026-10-19) — dialogo DIALOG, frame SSD, FULLSCREEN, HIDDEN/client-list **verificati**; B5 **non eseguito (xev muto)**, B6 **non eseguito (declinato)**; gate cargo restano **sospesi (no toolchain)**

## Appendice — output essenziali M1-residuals (verificato)

### A1–A2 sessione + root (`/tmp/ktile-ewmh-A.txt`)

```text
### A1 sessione
x11
:0
X-Cinnamon
### A2 root
_NET_SUPPORTING_WM_CHECK(WINDOW): window id # 0x2600006
_NET_WM_NAME:  not found.
_NET_CLIENT_LIST_STACKING(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
_NET_CURRENT_DESKTOP(CARDINAL) = 0
_NET_NUMBER_OF_DESKTOPS(CARDINAL) = 4
```

Nome WM da finestra check (non root, per protocollo EWMH):

```text
_NET_SUPPORTING_WM_CHECK(WINDOW): window id # 0x2600006
_MUFFIN_VERSION(UTF8_STRING) = "6.6.3"
_NET_WM_NAME(UTF8_STRING) = "Mutter (Muffin)"
```

### A4 strut: nessuna finestra root-child con `_NET_WM_STRUT_PARTIAL` >0 (`### A4 done` senza match, 73 figli)

### B1 dialogo Zenity (`/tmp/ktile-ewmh-B1.txt`)

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_WINDOW_TYPE(ATOM) = _NET_WM_WINDOW_TYPE_DIALOG
WM_CLASS(STRING) = "zenity", "Zenity"
WM_NAME(STRING) = "Informazioni"
_NET_WM_NAME(UTF8_STRING) = "Informazioni"
```

`WM_TRANSIENT_FOR` assente dal grep e da `xprop -id` diretto (verificato-assente).

### B2 Nemo SSD non-maximized (`/tmp/ktile-ewmh-B2.txt`)

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_FRAME_EXTENTS(CARDINAL) = 0, 0, 36, 0
WM_CLASS(STRING) = "nemo", "Nemo"
```

### B3 fullscreen (`/tmp/ktile-ewmh-B3.txt`) — tentativo Nemo vs retry Firefox

```text
# Nemo F11 (evidenza negativa: solo maximized)
_NET_WM_STATE(ATOM) = _NET_WM_STATE_MAXIMIZED_HORZ, _NET_WM_STATE_MAXIMIZED_VERT, _NET_WM_STATE_FOCUSED
_NET_WM_WINDOW_TYPE(ATOM) = _NET_WM_WINDOW_TYPE_NORMAL
# Firefox F11 (verificato)
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FULLSCREEN, _NET_WM_STATE_FOCUSED
_NET_WM_WINDOW_TYPE(ATOM) = _NET_WM_WINDOW_TYPE_NORMAL
```

### B4 minimize watcher (`/tmp/ktile-ewmh-B4.txt`)

```text
WID=0x5000006
--- t=1
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=2
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=3
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=4
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=5
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=6
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=7
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=8
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=9
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
--- t=10
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
--- CLIENT_LIST ---
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
```

### B5 xev (`/tmp/ktile-ewmh-B5.txt`, B5-bis `/tmp/ktile-ewmh-B5b.txt`) — non eseguito

```text
WID=0x5000006
```

Nessun evento in 2×15s (`timeout` exit 124, grep/head terminati vuoti).
