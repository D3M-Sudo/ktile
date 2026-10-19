# environment-baseline — Ktile M0

> Baseline read-only raccolta sulla macchina target il 2026-10-19.
> Legenda: **verificato** = osservato con comando · **ipotesi** = non ancora confermato.
> Regola M0: sola ispezione, nessun `sudo`, nessuna installazione di sistema (ADR-14).

## 1. Repository (verificato)

| Dato | Valore | Comando |
|---|---|---|
| Branch | `feature` | `git branch --show-current` |
| HEAD SHA | `1f6233c` (`1f6233ca748228c3a73190656b6fed21783716e1`) | `git rev-parse HEAD` |
| Working tree | pulito | `git status --porcelain` (vuoto) |
| Remote | `origin` → `https://github.com/D3M-Sudo/ktile.git`, branch `main/development/testing/feature` allineati | `git ls-remote --heads origin` |
| Commit base | `docs: project foundation (M0 docs, architecture, roadmap)` — 9 file, 899 righe | `git log --oneline -1` |

## 2. OS e hardware (verificato)

| Dato | Valore | Comando |
|---|---|---|
| OS | Linux Mint 22.3 (Zena), codename `noble` (base Ubuntu) | `cat /etc/os-release` |
| Arch | `x86_64` | `uname -m` |
| DE / WM | Cinnamon 6.6.9 / Muffin 6.6.3 (finestra WM: `Mutter (Muffin)` su `0x2600006`) | `cinnamon --version`, `muffin --version`, `xwininfo -root -children` |

## 3. Sessione grafica (verificato)

| Dato | Valore | Comando |
|---|---|---|
| `XDG_SESSION_TYPE` | `x11` ✅ (target richiesto) | `printenv XDG_SESSION_TYPE` |
| `DISPLAY` | `:0` | `printenv DISPLAY` |
| `XDG_CURRENT_DESKTOP` | `X-Cinnamon` | `printenv XDG_CURRENT_DESKTOP` |
| Monitor | 1× `eDP` 1920×1080+0+0 | `xrandr --listmonitors` |
| Geometria desktop | 1920×1080 | `_NET_DESKTOP_GEOMETRY` |

## 4. Toolchain (verificato)

| Tool | Esito | Note |
|---|---|---|
| `rustc` / `cargo` | ❌ **assenti** (`~/.cargo/bin` e `~/.rustup` inesistenti, `rustup` non trovato) | **Stop-condition M0 per M1**: riferire e attendere, NON installare. M0 resta completabile (read-only). |
| `python3` | ✅ 3.12.3 | `python3 --version` |
| `uv` / `uvx` | ✅ 0.10.7 in `/home/damiano/.local/bin/uv` | `uv --version` — policy ADR-14 applicabile |
| `xprop` | ✅ `/usr/bin/xprop` | diagnostica EWMH ok |
| `xwininfo` | ✅ `/usr/bin/xwininfo` | diagnostica finestre ok |
| `xev` | ✅ `/usr/bin/xev` | non usato in M0 (interattivo) |
| `xrandr` | ✅ `/usr/bin/xrandr` | monitor ok |

## 5. Matrice EWMH root (verificato)

`xprop -root _NET_SUPPORTING_WM_CHECK _NET_SUPPORTED _NET_WORKAREA _NET_NUMBER_OF_DESKTOPS _NET_CURRENT_DESKTOP _NET_DESKTOP_GEOMETRY _NET_CLIENT_LIST _NET_CLIENT_LIST_STACKING _NET_ACTIVE_WINDOW`

| Proprietà | Valore osservato | Note |
|---|---|---|
| `_NET_SUPPORTING_WM_CHECK` | `0x2600006` (titolo `Mutter (Muffin)`) | Doppia verifica root→WM ok (D6 soddisfatto: EWMH essenziale presente) |
| `_NET_SUPPORTED` | ~85 atomi: `WM_NAME`, `CLOSE_WINDOW`, `WM_STATE*`, `CURRENT_DESKTOP`, `CLIENT_LIST(+STACKING)`, `ACTIVE_WINDOW`, `MOVERESIZE_WINDOW`, `FRAME_EXTENTS`, `STRUT(+PARTIAL)`, `WORKAREA`, `DESKTOP_GEOMETRY/VIEWPORT`, `_GTK_FRAME_EXTENTS`, … | Ampia copertura Muffin; nessuna whitelist nomi (D6) |
| `_NET_NUMBER_OF_DESKTOPS` | 4 | — |
| `_NET_CURRENT_DESKTOP` | 0 | — |
| `_NET_DESKTOP_GEOMETRY` | 1920, 1080 | — |
| `_NET_WORKAREA` | `0,0,1920,1040` ×4 desktop | Pannello 40px in basso (1080−1040); solo controllo incrociato (ADR-12) |
| `_NET_CLIENT_LIST` / `_STACKING` | `0x2a00021, 0x5200017, 0x5400004` (identiche) | Entrambe presenti, nessun fallback necessario |
| `_NET_ACTIVE_WINDOW` | `0x5400004` (devin-desktop, focused) | Coerente con `_NET_WM_STATE_FOCUSED` |

## 6. Campioni finestre (verificato)

| Finestra | Classe / tipo | Stato | Note per eleggibilità |
|---|---|---|---|
| `0x2a00021` Desktop | `nemo-desktop`, `DESKTOP` | `SKIP_PAGER, SKIP_TASKBAR, STICKY`, no `_NET_WM_DESKTOP` | Esclusa (non-NORMAL) |
| `0x5200017` Firefox | `Navigator/firefox`, `NORMAL`, desktop 0 | `MAXIMIZED_HORZ+VERT`, `WM_TRANSIENT_FOR` assente, `_GTK_FRAME_EXTENTS=0,0,0,0`, min-size 638×120 | Eleggibile (maximized → policy M4/M5); CSD GTK senza extents residui |
| `0x5400004` devin-desktop | `devin-desktop`, `NORMAL`, desktop 0 | `MAXIMIZED_HORZ+VERT, FOCUSED` | Eleggibile, finestra attiva |
| Pannello Cinnamon | `cinnamon/Cinnamon` (`0x260000b`, `0x2600001`) | **nessuno strut** (`_NET_WM_STRUT_PARTIAL` assente) | ⚠️ Ipotesi da chiudere in M1: pannello non in `_NET_CLIENT_LIST`, strut non trovato su queste finestre — verificare quale finestra espone lo strut da 40px (o dedurre area da `_NET_WORKAREA` come fallback documentato) |

## 7. Stop-condition M0

| Condizione | Esito |
|---|---|
| Working tree pulito | ✅ pulito su `feature` |
| Sessione X11 | ✅ `x11`, `:0` |
| Rust presente | ❌ **assente** → stop per M1 (riferito, in attesa istruzioni; NON installato per policy) |
| Repository presente | ✅ `feature @ 1f6233c` |

**Verdetto M0: baseline completata e documentata; gate M1 bloccato finché Rust non è disponibile.**

## 8. Ipotesi doc 03 §9 — stato dopo M0

1. Geometrie richieste applicate da Muffin — **ipotesi** (serve M5).
2. `_NET_WORKAREA` con pannello laterale/auto-hide — **ipotesi** (solo pannello basso 40px verificato).
3. Messaggi client `_NET_WM_STATE` — **ipotesi**.
4. `_GTK_FRAME_EXTENTS` su CSD — parziale: Firefox CSD → `0,0,0,0` **verificato** su 1 campione.
5. Sequenza Map/Unmap minimize/workspace — **ipotesi** (serve M1: osservazione 30s).
6. Strut auto-hide real-time — **ipotesi** (+ punto §6: quale finestra espone lo strut).
7. Workspace/viewport default — parziale: 4 desktop, viewport non letta → **ipotesi** residua.
8. Nome WM — **verificato**: `Mutter (Muffin)` su `0x2600006`.
9. Stabilità `_NET_ACTIVE_WINDOW` — parziale: coerente su 1 lettura → **ipotesi** su alt-tab.
10. Interazione "edge tiling" Cinnamon — **ipotesi**.
