# M1-findings — Ktile M1-prep + M1-residuals + Cloud Gate M1/M2 (2026-10-19) + Osservazioni target B5/B6 (2026-10-20)

> **Stato M1 in Cloud**: Gate M1 sbloccato e chiuso in ambiente cloud con toolchain Rust 1.94.0 (ADR-15).
> Esiti build/clippy/test verdi in cloud; test d'integrazione eseguiti sotto Xvfb + openbox.
> **Importante**: Gli esiti X11 registrati in cloud sono contrassegnati come *"verificato su WM di test (openbox), da confermare su Muffin"*.
> Residui B5 e B6 trasferiti all'utente tramite il protocollo `docs/milestones/M1-residuals-user.md`.
> **2026-10-20 — M1 chiuso anche sul target**: B5 (obbligatoria) e B6 (opzionale, consenso utente) eseguite su Mint 22.3 / Cinnamon 6.6.9 / Muffin 6.6.3, X11 `:0` — vedi **§2c**; gate read-only rieseguito (0 match geometrici). HEAD al momento delle osservazioni: `99cda17`.

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

## 2c. Osservazioni runtime su Muffin (target, 2026-10-20)

> Eseguite sulla macchina target: Linux Mint 22.3 / Cinnamon 6.6.9 / Muffin 6.6.3, sessione X11 `:0`, 2026-10-20 00:13–00:33.
> Stato git in fase 0: `feature` @ `99cda17`, `git status --porcelain` vuoto (ripetuto al termine delle osservazioni: invariato).
> Strumenti: `xdotool` ✅ `/usr/bin/xdotool` · `xev` ✅ `/usr/bin/xev` · `xprop` ✅ `/usr/bin/xprop` · `xwininfo` ✅ `/usr/bin/xwininfo` — tutti presenti; `cargo`/`rustc` ❌ assenti (ADR-15).
> Vincoli rispettati: zero installazioni, zero sudo, zero scritture geometriche (`grep -rniE 'configure_window|moveresize' src/ tests/` → **0 match**, rieseguito al termine), finestre toccate solo da minimize/restore **manuale dell'utente** (B5) e da modifica impostazioni pannello con revert verificato (B6).
> Nota protocollo: il fallback WID (`awk -F'[:,] ' '{print $2}'`) non estrae l'hex (tentativo 1); usato il WID identificato e verificato via `WM_CLASS`.

### B5 — sequenza Unmap/HIDDEN su Muffin (obbligatoria) — **verificato**

Finestra campione: `0x5000006` = `gnome-terminal-server` / `"Terminale"`, `_NET_WM_WINDOW_TYPE_NORMAL`, `_NET_WM_DESKTOP = 0`.

**(a) Tentativo 1 (00:13) — fallito, evidenza conservata**: l'ID estratto contiene la stringa completa `window id # 0x5000006` e xprop lo rifiuta; tutti i 12 tick hanno prodotto lo stesso errore.

```text
=== B5 WATCHER — 2026-10-20 00:13:43 ===
WID=window id # 0x5000006
=== T0 00:13:43 — stato iniziale ===
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=1s 00:13:44 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=2s 00:13:45 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=3s 00:13:46 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=4s 00:13:47 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=5s 00:13:48 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=6s 00:13:49 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=7s 00:13:50 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=8s 00:13:51 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=9s 00:13:52 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=10s 00:13:53 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=11s 00:13:54 ---
xprop: error: Invalid window id format: window id # 0x5000006.
--- t=12s 00:13:55 ---
xprop: error: Invalid window id format: window id # 0x5000006.
=== 00:13:55 — stato finale finestra ===
xprop: error: Invalid window id format: window id # 0x5000006.
=== _NET_CLIENT_LIST (root) ===
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
=== _NET_ACTIVE_WINDOW (root) ===
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
=== FINE WATCHER ===
```

**(b) Watcher 12 s — RUN 2 (00:15:25–00:15:38) — ciclo completo minimize/restore — verificato**

```text
=== B5 WATCHER run2 — 2026-10-20 00:15:25 ===
WID=0x5000006 (gnome-terminal-server 'Terminale', type NORMAL, desktop 0)
=== T0 00:15:25 ===
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5000006
--- t=1s 00:15:27 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5000006
--- t=2s 00:15:28 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5000006
--- t=3s 00:15:29 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5000006
--- t=4s 00:15:30 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
--- t=5s 00:15:31 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
--- t=6s 00:15:32 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
--- t=7s 00:15:33 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
--- t=8s 00:15:34 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
--- t=9s 00:15:35 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5400004
--- t=10s 00:15:36 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5000006
--- t=11s 00:15:37 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5000006
--- t=12s 00:15:38 ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
_NET_ACTIVE_WINDOW(WINDOW): window id # 0x5000006
=== stato finale ===
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 0
=== _NET_CLIENT_LIST finale ===
_NET_CLIENT_LIST(WINDOW): window id # 0x2a00021, 0x5200017, 0x5400004, 0x5000006
=== FINE ===
```

**(c) xev 20 s — secondo minimize/restore (00:16:42) — verificato**

`grep -nE 'UnmapNotify|MapNotify|DestroyNotify'` sull'output grezzo → **0 match**. Output grezzo completo:

```text
=== B5 xev RUN — 2026-10-20 00:16:42 WID=0x5000006 ===
--- stato pre-xev ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED

LeaveNotify event, serial 18, synthetic NO, window 0x5000006,
    root 0x3cc, subw 0x0, time 38017652, (99,-2), root:(487,157),
    mode NotifyNormal, detail NotifyAncestor, same_screen YES,
    focus NO, state 16

PropertyNotify event, serial 18, synthetic NO, window 0x5000006,
    atom 0x153 (WM_STATE), time 38019192, state PropertyNewValue

PropertyNotify event, serial 18, synthetic NO, window 0x5000006,
    atom 0x130 (_NET_WM_STATE), time 38019192, state PropertyNewValue

PropertyNotify event, serial 18, synthetic NO, window 0x5000006,
    atom 0x15e (_GTK_EDGE_CONSTRAINTS), time 38019192, state PropertyNewValue

FocusOut event, serial 18, synthetic NO, window 0x5000006,
    mode NotifyNormal, detail NotifyNonlinearVirtual

PropertyNotify event, serial 21, synthetic NO, window 0x5000006,
    atom 0x130 (_NET_WM_STATE), time 38019205, state PropertyNewValue

PropertyNotify event, serial 21, synthetic NO, window 0x5000006,
    atom 0x15e (_GTK_EDGE_CONSTRAINTS), time 38019205, state PropertyNewValue

PropertyNotify event, serial 21, synthetic NO, window 0x5000006,
    atom 0x153 (WM_STATE), time 38027276, state PropertyNewValue

PropertyNotify event, serial 21, synthetic NO, window 0x5000006,
    atom 0x130 (_NET_WM_STATE), time 38027276, state PropertyNewValue

PropertyNotify event, serial 21, synthetic NO, window 0x5000006,
    atom 0x15e (_GTK_EDGE_CONSTRAINTS), time 38027276, state PropertyNewValue

FocusIn event, serial 21, synthetic NO, window 0x5000006,
    mode NotifyNormal, detail NotifyNonlinear

KeymapNotify event, serial 21, synthetic NO, window 0x0,
    keys:  48  0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   
           0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   

PropertyNotify event, serial 21, synthetic NO, window 0x5000006,
    atom 0x130 (_NET_WM_STATE), time 38027289, state PropertyNewValue

PropertyNotify event, serial 21, synthetic NO, window 0x5000006,
    atom 0x15e (_GTK_EDGE_CONSTRAINTS), time 38027289, state PropertyNewValue

FocusOut event, serial 21, synthetic NO, window 0x5000006,
    mode NotifyNormal, detail NotifyInferior

EnterNotify event, serial 21, synthetic NO, window 0x5000006,
    root 0x3cc, subw 0x0, time 38028610, (695,500), root:(1083,659),
    mode NotifyNormal, detail NotifyNonlinear, same_screen YES,
    focus NO, state 16

KeymapNotify event, serial 21, synthetic NO, window 0x0,
    keys:  48  0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   
           0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   
--- stato post-xev ---
_NET_WM_STATE(ATOM) = _NET_WM_STATE_FOCUSED
```

Maschera verificata dalla sorgente di xev: il default event mask è `... StructureNotifyMask | SubstructureNotifyMask | SubstructureRedirectMask | FocusChangeMask | PropertyChangeMask ...` (lista confermata dal patch xorg-devel "[PATCH xev] Add a -event parameter to control the event mask", il cui man page recita "When not specified, all events are selected") → l'assenza di Unmap/Map/Destroy **non** è un falso negativo da maschera mancante.

#### Le 4 domande B5

| # | Domanda | Esito | Evidenza (raw sopra) |
|---|---|---|---|
| 1 | `HIDDEN` compare come valore di `_NET_WM_STATE`? | **SÍ — verificato** | `_NET_WM_STATE(ATOM) = _NET_WM_STATE_HIDDEN` ai tick t=4–9 s del RUN 2 |
| 2 | La proprietà viene eliminata/impoverita al minimize? | **Non eliminata: proprietà sempre presente, valore sostituito — verificato** | da `_NET_WM_STATE_FOCUSED` (t=0–3 s) a `_NET_WM_STATE_HIDDEN` (t=4–9 s, FOCUSED rimosso) e torna a FOCUSED (t=10–12 s); `PropertyNotify` su `_NET_WM_STATE` e `WM_STATE` (xev, time 38019192 / 38027276); mai proprietà assente |
| 3 | `UnmapNotify` osservato? | **NO — non osservato su 2 cicli completi — verificato** | grep vuoto su 62 righe xev durante minimize **e** ripristino; transizione segnalata solo da `PropertyNotify(WM_STATE, _NET_WM_STATE)` + `FocusOut`/`FocusIn`. Muffin non unmap/rimappa al minimize |
| 4 | La finestra resta in `_NET_CLIENT_LIST` mentre è minimizzata? | **SÍ — verificato** | `0x5000006` presente a **ogni** tick incluso t=4–9 s (stato HIDDEN); membership invariata per tutti i 12 s |

Corollari verificati: `_NET_WM_DESKTOP` = 0 costante; durante l'`HIDDEN` `_NET_ACTIVE_WINDOW` = `0x5400004` e al ripristino torna `0x5000006`.

#### Tabella ipotesi doc 03 §9 — aggiornamento (osservazioni 2026-10-20)

| Ipotesi (doc 03 §9) | Esito | Nota |
|---|---|---|
| n.2 — `_NET_WORKAREA` con pannello laterale/auto-hide | **aggiornata (B6): "pannello laterale" chiuso**; auto-hide residuo | laterale `40,0,1880,1080` ×4 entro ≤1 s; revert = baseline. Variante auto-hide **non testata** → residuo M6/M7 |
| n.5 — sequenza Map/Unmap per minimize/cambio workspace | **chiusa per "minimize" — verificato** | nessun `UnmapNotify`/`MapNotify`/`DestroyNotify` su 2 cicli; solo `PropertyNotify`. Parte "cambio workspace" non osservata (residuo minore) |
| n.6 — `_NET_WM_STRUT_PARTIAL` in tempo reale con auto-hide | **aggiornata — verificato (variante pannello laterale)** | scansione su **tutte** le finestre root: **nessuna** espone `_NET_WM_STRUT(_PARTIAL)` (prima e durante il laterale); `_NET_WORKAREA` si aggiorna comunque. Auto-hide non testato → residuo M6/M7 |
| n.9 — stabilità `_NET_ACTIVE_WINDOW` su alt-tab/switcher | **aggiornata (parziale)** | `_NET_ACTIVE_WINDOW` segue correttamente il focus durante minimize/restore (`0x5000006`↔`0x5400004`); alt-tab/switcher **non testato** → ipotesi residua |
| n.10 — edge tiling Cinnamon vs finestre Ktile | **ipotesi (non osservata)** | fuori scope B5/B6; serve M5 |

Bonus: punto aperto baseline M0 §6 ("quale finestra espone lo strut da 40 px") → **chiuso: NESSUNA finestra** (scansione completa dei figli root, 0 match).

### B6 — `_NET_WORKAREA` con pannello laterale + revert (opzionale; consenso esplicito dell'utente) — **verificato**

**(a) Baseline (00:20:06):**

```text
=== B6 BASELINE 2026-10-20 00:20:06 ===
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
_NET_DESKTOP_GEOMETRY(CARDINAL) = 1920, 1080
--- strut pannelli (da baseline M0) ---
_NET_WM_STRUT_PARTIAL:  not found.
_NET_WM_STRUT:  not found.
_NET_WM_STRUT_PARTIAL:  not found.
_NET_WM_STRUT:  not found.
```

**(b) Scansione strut su tutti i figli root, pannello in basso — nessun match:**

```text
FINE_SCAN_00:22:37
```

**(c) Transizione basso → sinistra catturata dal vivo (00:30:06, t=14 s; entro ≤1 s) + steady 10 s + scansione strut laterale:**

```text
=== B6 WATCHER LATERALE v2 — start 2026-10-20 00:29:51 ===
--- attesa t=1s 00:29:52 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=2s 00:29:53 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=3s 00:29:54 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=4s 00:29:55 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=5s 00:29:56 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=6s 00:29:57 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=7s 00:29:58 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=8s 00:29:59 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=9s 00:30:00 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=10s 00:30:01 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=11s 00:30:03 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=12s 00:30:04 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=13s 00:30:05 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- attesa t=14s 00:30:06 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
*** CAMBIO RILEVATO t=14s 00:30:06 ***
=== STATO LATERALE STEADY — 10 s ===
--- steady t=1s 00:30:07 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=2s 00:30:08 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=3s 00:30:09 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=4s 00:30:10 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=5s 00:30:11 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=6s 00:30:12 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=7s 00:30:13 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=8s 00:30:14 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=9s 00:30:15 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- steady t=10s 00:30:16 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- STRUT SCAN con pannello laterale ---
FINE_LATERALE_V2 00:30:16
```

**(d) Transizione revert sinistra → basso catturata dal vivo (00:33:04, t=12 s) + verifica stabile 5 s + lettura finale:**

```text
=== B6 REVERT v2 — start 2026-10-20 00:32:52 ===
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=1s 00:32:53 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=2s 00:32:54 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=3s 00:32:55 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=4s 00:32:56 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=5s 00:32:57 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=6s 00:32:58 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=7s 00:32:59 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=8s 00:33:00 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=9s 00:33:01 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=10s 00:33:02 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=11s 00:33:03 ---
_NET_WORKAREA(CARDINAL) = 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080, 40, 0, 1880, 1080
--- attesa t=12s 00:33:04 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
*** TRANSIZIONE REVERT t=12s 00:33:04 ***
=== VERIFICA STABILE — 5 s ===
--- verifica t=1s 00:33:05 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- verifica t=2s 00:33:06 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- verifica t=3s 00:33:07 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- verifica t=4s 00:33:08 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- verifica t=5s 00:33:09 ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
--- lettura finale ---
_NET_WORKAREA(CARDINAL) = 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040, 0, 0, 1920, 1040
BASELINE_ATTESA = 0, 0, 1920, 1040 x4
FINE_REVERT_V2 00:33:09
```

| Voce B6 | Esito |
|---|---|
| Variazione area | **verificato**: `0,0,1920,1040` → `40,0,1880,1080` su tutti e 4 i desktop (riserva 40 px a sinistra, altezza piena 1080), entro ≤1 s dal cambio |
| Comportamento struts / workarea | **verificato**: nessun cliente X espone strut (finestre root scansionate, pannello in basso **e** laterale); `_NET_WORKAREA` si aggiorna in tempo reale; `_NET_DESKTOP_GEOMETRY` invariata `1920,1080` |
| Esito revert | **verificato**: `40,0,1880,1080` → `0,0,1920,1040` ×4 entro ≤1 s; 5 tick stabili + lettura finale **identica alla baseline**; pannello lasciato nella posizione originale (basso) |

Impatto ADR-12 (nota — **ADR non modificato**): su questo target la fonte primaria "strut + RandR" non è alimentata da alcuno strut EWMH e l'unico segnale di riserva è `_NET_WORKAREA`, risultato accurato e real-time su singolo monitor → **ADR-12 da re-verificare** (registrato in ROADMAP §3).

### Anomalie operative della sessione (2026-10-20)

1. **Fallback WID del protocollo non funzionante**: `awk -F'[:,] ' '{print $2}'` restituisce `window id # 0x5000006` → tentativo 1 invalido (evidenza (a)); usato il WID verificato via `WM_CLASS`.
2. **Primo watcher B6 andato vuoto** (00:24:31): spostamento pannello non completato entro la finestra di osservazione → `_NET_WORKAREA` costante sulla baseline per tutti i 10 s (tentativo non valido, non usato come esito).
3. **Poller di rilevamento cambio non persistito**: script multi-linea con escape `\'` dentro apici singoli → quoting rotto, output su terminale invece che su file; processo terminato senza residui (`pgrep -af` pulito). Rifatto con watcher foreground ((c)/(d)).
4. **Ritardi di sincronizzazione utente**: revert completato prima dell'avvio del watcher (00:28:23) e pannello già laterale all'avvio successivo (00:31:55) → entrambi i tentativi scartati; ciclo ripetuto con watcher a rilevamento cambio, catturando entrambe le transizioni dal vivo.
5. **Prima scansione strut con regex errata**: `grep -oE '0x[0-9a-f]+'` catturava numeri di geometria (es. `0x1040` da `1920x1040`) → rifatta estraendo la colonna `$1` delle righe figlio di `xwininfo -root -children`.
6. **`cargo`/`rustc` assenti sul target**: `cargo test` non eseguibile → conteggio test verificato staticamente (vedi §4).

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
  `grep -rniE 'configure_window|moveresize' src/ tests/` → **0 match (esattamente zero corrispondenze)** — **rieseguito sul target 2026-10-20: `GREP_EXIT=1 MATCHES=0` (0 match, invariato)**.
- `cargo fmt --check`: **passato (zero diff)** — cloud 2026-10-19 (non rieseguibile sul target: `cargo` assente, ADR-15).
- `cargo clippy --all-targets -- -D warnings`: **0 avvisi / 0 errori** — cloud 2026-10-19.
- `cargo test`: **15 test passati, 0 falliti** = 9 (`tests/eligibility.rs`) + 4 (`tests/exit_codes.rs`) + 2 (`tests/geometry.rs`). I 5 test di `tests/x11_integration.rs` sono **tutti `#[ignore]`** e scattano solo con `cargo test --ignored` (Xvfb + openbox).
- **Correzione conteggio** (la dicitura "15 test passati" riferiva alla sola suite plain): **15 = `cargo test` senza flag; 20 = totale sommato dall'appendice §5 (9+4+2+5)** includendo la suite ignored. Verifica statica sul target 2026-10-20 (`cargo` assente): `grep -rn '#\[test\]' src/ tests/` → **20** occorrenze (9+4+2+5; zero test in `src/`); `grep -rn '#\[ignore' tests/` → **5**, tutte in `tests/x11_integration.rs` → coerente con l'appendice cloud.
- **Gate M1 CHIUSO anche sul target: 2026-10-20** — residui B5/B6 eseguiti e registrati (§2c), zero scritture geometriche, zero installazioni, nessun file fuori scope (working tree pulito su `feature` @ `99cda17` al termine delle osservazioni).

## 5. Appendice: Output e Prove Rilevate

> Nota 2026-10-20: gli output grezzi completi delle osservazioni target B5/B6 sono nella §2c.

### Command Cargo Check & Test Output
```text
running 9 tests in tests/eligibility.rs ... 9 passed
running 4 tests in tests/exit_codes.rs ... 4 passed
running 2 tests in tests/geometry.rs ... 2 passed
running 5 tests in tests/x11_integration.rs ... 5 passed (Xvfb + openbox)
```
