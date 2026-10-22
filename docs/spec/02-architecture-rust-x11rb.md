# 02 — Architettura e stack (Rust + x11rb)

## 1. Stack: decisioni prese

- **Linguaggio: Rust** — processo residente a basso consumo, singolo binario, errori espliciti. Validazione di RAM/CPU prevista in M6 su Mint (nessuna soglia numerica pre-impostata).
- **Backend X11: `x11rb` — SCELTO** (confronto comparativo eseguito in analisi: API safe e tipizzate, errori come `Result` su cookie/reply, zero dipendenze native, manutenzione attiva). Alternative valutate e scartate: `xcb` (FFI a libxcb, build system-dependent) e `x11-dl` (Xlib dinamica, superficie FFI più ampia). **Vietato aggiungere più backend.**
- `x11rb` non fornisce helper EWMH di alto livello: il modulo `x11/ewmh.rs` è codice nostro, piccolo e testabile — è la superficie unica da validare contro Muffin.

## 2. Crate

| Crate | Quando | Note |
|---|---|---|
| `x11rb` | M1 | Backend unico; estensione `randr` per i monitor (M4+) |
| `thiserror` | M1 | Errori tipizzati → codici d'uscita |
| `serde` + `derive`, `toml` | M4 | Config |
| `regex` | M4 | `title_regex` e match class negli escludi |
| `clap` + `derive` | M4 | CLI e help |
| `tracing`/`tracing-subscriber` | M6 (gate) | Solo se stderr manuale non basta |
| `signal-hook` | M6 | Shutdown cooperativo su SIGTERM/SIGINT |
| `proptest` | post-M3 (gate) | Property test su sequenze di inserimento |

Escluse: `xcb`, `x11-dl`, dbus/zbus, crate di rete, tray, GUI, animazioni.

## 3. Componenti e confini

**Stato effettivo al 2026-10-22:** M1–M4 sono implementate e M5 ha un applier manuale opt-in (`--allow-apply`) con gate live e readback. Il daemon/event loop, il piano incrementale basato sull'albero persistente e il coalescing degli eventi non sono ancora implementati: restano obiettivi M6, non descrivere il diagramma seguente come flusso già attivo.

```text
CLI / config (cli.rs, config.rs)
    |
    v
main.rs -> config validation -> X11 connect + EWMH checks
    |
    v
snapshot client/workspace/area (x11/ewmh, monitor, icccm)
    |
    v
eligibility + reconciliation read-only (state/)
    |
    v
layout/ pure geometry + plan
    |
    v
[default / --dry-run] stampa piano, nessuna richiesta di scrittura
[--allow-apply] x11/applier.rs -> gate live -> _NET_MOVERESIZE_WINDOW -> readback/report

FUTURO M6 (non implementato): eventi X11 -> daemon -> aggiornamento stato e piano incrementale
```

Regole dei confini:

1. **`layout/` non conosce X11**: input astratto (`WorkspaceArea`, `Vec<WindowSlot>`, `LayoutOptions`), output `Vec<(WindowId, Rect)>`. `WindowId` è un newtype opaco.
2. **`state/` non parla col server**: riceve snapshot da `x11/` come tipi di dominio (`WindowMeta`, `Monitor`, `Workspace`).
3. **`x11/` è l'unico strato che tocca la connessione**; errori tipizzati (`ConnectionLost`, `BadWindow(id)`, `EwmhMissing(atom)`).
4. **`applier` è l'unico punto che invia geometrie**; l'applicazione è disattivata di default e richiede `--allow-apply`; ogni richiesta passa il gate live e produce esito/readback.
5. **Event loop (M6, non implementato)** dovrà tradurre eventi X11 in eventi di dominio e guidare riconciliazioni selettive; il codice corrente esegue un singolo snapshot/piano per invocazione CLI.
6. **Shutdown**: stop cooperativo, nessuna richiesta ri-inviata, connessione chiusa; Ktile non "ripristina" nulla all'uscita.

## 4. Struttura moduli

```text
src/
  main.rs                  wiring, codici uscita
  cli.rs                   clap: --check-config --list-windows --dry-run
                           --layout L --allow-apply --daemon --config PATH --version
  config.rs                TOML -> Config validata (serde), limiti, defaults
  errors.rs                KtileError -> exit code
  x11/
    mod.rs
    connection.rs          connect, intern atoms, extension checks, retry
    events.rs              maschere eventi, polling, DomainEvent
    ewmh.rs                lettura proprietà EWMH
    icccm.rs               WM_CLASS, transient, normal hints (sola lettura)
    monitor.rs             RandR heads, workarea, strut -> WorkspaceArea
    applier.rs             _NET_MOVERESIZE_WINDOW, safety gate, frame-aware readback
  state/
    registry.rs            WindowId -> WindowState (Active|Suspended|Excluded|Gone)
    eligibility.rs         policy di esclusione
    workspace.rs           WorkspaceId = (desktop, monitor); stato layout
    reconciliation.rs      diff snapshot<->stato, ricalcolo selettivo
  layout/
    mod.rs                 trait Layout { plan(area, slots, opts) -> Plan }
    geometry.rs            Rect, Area, split helpers, clamp, invarianti
    tree.rs                albero binario dwindle: insert/remove/promote
    dwindle.rs             policy split, ratio, gaps
    master_stack.rs        master selection, promozione, mfact
    plan.rs                Plan { moves }, diff/idempotenza
tests/
  layout_tree.rs           no X11
  layout_master.rs
  geometry_edge.rs         aree piccole, gap estremi, coordinate negative
  eligibility.rs
  fixtures/
```

I file vengono creati solo quando autorizzati dalla milestone attiva.

## 5. Event loop e stato

Bootstrap:
1. verificare `DISPLAY` e connessione;
2. leggere `_NET_SUPPORTING_WM_CHECK` (doppia verifica root+finestra WM) e registrare il nome;
3. leggere lista client EWMH (`_NET_CLIENT_LIST`, fallback stacking con warn);
4. leggere le proprietà necessarie per ogni finestra;
5. calcolare lo stato iniziale; non spostare nulla in `--dry-run`.

Loop: `MapNotify`, `UnmapNotify`, `DestroyNotify`, `PropertyNotify` (root e finestre tracked), eventi RandR; filtro anti-feedback sui propri `ConfigureNotify`; riconciliazione completa solo al bootstrap e dopo errori significativi. Coalescing degli eventi ravvicinati, mai polling continuo.

## 6. Eventi e proprietà (riepilogo)

- `MapNotify`/`UnmapNotify`/`DestroyNotify`: ciclo di vita (`Suspended` ≠ `Gone`, decisione D8).
- `ConfigureNotify`: osservazione geometrie reali; filtro in-flight.
- `PropertyNotify`: rileggere sempre la proprietà (l'evento non contiene il valore).
- Focus: `_NET_ACTIVE_WINDOW` su root come fonte primaria; `FocusIn/Out` solo segnale secondario.
- `_NET_WM_STATE`, `_NET_WM_WINDOW_TYPE`, `WM_TRANSIENT_FOR`, `WM_NORMAL_HINTS` (sola lettura): eleggibilità e vincoli.
- Su Muffin/Mutter: **non riscrivere mai `WM_NORMAL_HINTS`** (vincolo emerso dall'analisi di Cortile, `Compatible("icccm.SizeHintPMinSize")`; da re-verificare sul target — vedi doc 03).

## 7. Geometria

Tipo interno indipendente da X11:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect { pub x: i32, pub y: i32, pub width: u32, pub height: u32 }
```

Regole: gaps `outer` applicato una volta sull'area di lavoro; `inner` tra le metà di ogni split (decisione D2: chiave unica `outer`); distribuzione dei pixel residui documentata nei test; niente offset fissi per pannelli (strut, doc 03); conversioni con `saturating_*`/`checked_*`; aree piccole → clamp + log, mai panic.

## 8. Layout

Core (D5): **dwindle** (albero binario persistente; inserimento in `second`; foglia target = attiva se tiled, altrimenti MRU; `split_mode = longest-side|alternating`; `split_ratio` default 0.5, clamp [0.25,0.75]) e **master-stack** (`new_status = slave|master` default `slave` per D7; promozione dello slave più vecchio; `mfact` default 0.55, clamp [0.25,0.75]).

vertical/horizontal/grid restano solo casi di test del motore in M2, non feature utente della v1.

## 9. Applicazione delle geometrie

Prima di spostare: finestra ancora esistente, eleggibile, non fullscreen, dimensioni positive, rispetto dei minimi ICCCM (lettura) — mai hint riscritti. Preferire messaggi client EWMH dove la specifica li definisce; testare l'effettivo comportamento di Muffin (M5). Leggere il risultato reale e non insistere: max 3 richieste per burst, poi deviazione loggata.

## 10. Daemon e CLI

```text
ktile --version
ktile --check-config
ktile --list-windows
ktile --dry-run --layout dwindle|master-stack
ktile --layout dwindle|master-stack
ktile --daemon        (foreground, opt-in; niente PID file, niente fork)
```

Codici d'uscita: 0 successo (anche "nessuna finestra eleggibile", D4), 2 argomenti invalidi, 3 config invalida, 4 sessione X11 non disponibile, 5 errore X11/EWMH, 6 EWMH essenziale assente (D6: nessuna whitelist di nomi WM). Foreground gestito da systemd --user o autostart Cinnamon; `--daemon` non implica distacco.

## 11. Dipendenze e build

Aggiungere una crate alla volta: licenza, feature, dipendenze transitive, compatibilità toolchain, `Cargo.lock` versionato. Comandi:

```bash
cargo new ktile
cd ktile
cargo add x11rb
cargo add thiserror
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

## 12. Stato per layout a suddivisioni

Motore ad albero (schema concettuale):

```rust
enum Node {
    Split { axis: Axis, ratio_milli: u16, first: Box<Node>, second: Box<Node> },
    Leaf(WindowId),
}
enum Axis { Horizontal, Vertical }
```

Albero **persistente** (equivalente a `preserve_split=true` di Hyprland): direzione e rapporto di ogni split restano memorizzati. Regole esplicite: inserimento, rimozione con promozione del fratello, ripristino sospese (D8), minimi dimensionali, gaps. Test del motore senza connessione X11.

## 13. Limiti del riferimento Hyprland

Hyprland gestisce compositor, rendering, animazioni e input nella propria architettura Wayland. Ktile può riprodurre tiling, gaps e convenzioni UX, non effetti grafici né eventi pointer-aware. Nessuna dipendenza da API Hyprland/Wayland.


## 5. Divergenza da risolvere: percorso del file di configurazione

`src/main.rs` cerca prima `./ktile.toml`, poi `~/.config/ktile/ktile.toml`; se non trova nessun file usa i default, mentre `--config PATH` esplicito senza file fallisce. Alcune specifiche storiche indicano `~/.config/ktile/config.toml` e la configurazione esempio è chiamata `config.example.toml`. Questa differenza è stata verificata direttamente nel codice ma non va risolta aggiornando unilateralmente i documenti: stabilire prima il percorso canonico voluto e poi aggiornare implementazione, esempi e tutte le specifiche in modo coerente.
