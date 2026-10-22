# 01 — Panoramica progetto (Ktile)

## 1. Scopo

**Nome definitivo: `Ktile`** (decisione D1, 2026-10-17). Binario: `ktile`. Configurazione: `${XDG_CONFIG_HOME:-~/.config}/ktile/config.toml`.

Ktile è un tiler X11 leggero per Linux Mint Cinnamon, progettato per funzionare **accanto a Muffin**, senza sostituire il window manager. Il riferimento d'esperienza è Hyprland: tiling dinamico con layout a suddivisioni binarie (**dwindle**) e **master-stack**, gaps, finestra attiva ben distinguibile e finestre flottanti per le categorie appropriate. L'implementazione resta un client X11 esterno, non un compositor Wayland. Offre modalità manuali e auto-tiling opt-in, con comportamento prevedibile e recupero sicuro.

### Obiettivi misurabili

- Supportare Linux Mint Cinnamon su sessione **X11**; rilevare e rifiutare chiaramente sessioni Wayland non supportate.
- Non richiedere privilegi root, servizi di sistema o dipendenze di desktop non necessarie.
- RAM e CPU inattive contenute: nessun polling ad alta frequenza; eventi X11 e riconciliazioni limitate.
- Non interferire con finestre minimizzate, dialoghi, popup, finestre sticky, fullscreen o finestre escluse.
- Non coprire pannelli Cinnamon né aree riservate (`_NET_WM_STRUT_PARTIAL`, `_NET_WORKAREA` solo come controllo incrociato).
- Configurazione leggibile, log diagnostici, modalità `--dry-run`/`--check-config`.
- Test unitari per la geometria; test d'integrazione separati per X11 reale.

## Drift da risolvere: percorso canonico della configurazione

Questa panoramica conserva la decisione progettuale `~/.config/ktile/config.toml`, ma il codice corrente (`src/main.rs`) cerca `./ktile.toml` e poi `~/.config/ktile/ktile.toml`; se non trova un file usa i default in memoria. Il percorso canonico è quindi **non allineato**: non assumere che codice o documento rappresentino automaticamente l'intento corretto. Prima di cambiare uno dei due, confermare il contratto desiderato e poi sincronizzare codice, esempio TOML, README e CLI docs.

## 2. Fuori scope iniziale

- Sostituire Muffin o implementare un window manager completo.
- Supporto Wayland, compositing, workspace avanzate, manipolazione di contenuti delle finestre.
- Hotkey globali implementate con polling o intercettazione invasiva: le scorciatoie si configurano in Cinnamon che esegue i comandi CLI.
- Modifiche a pannelli Cinnamon, temi, decorazioni o configurazione di sistema.
- Avvio automatico obbligatorio: scelta dell'utente tramite autostart (M8).
- Replica di effetti grafici Hyprland (animazioni, blur, bordi colorati): responsabilità del compositor/WM, non di un tiler esterno.

## 3. Evoluzione rispetto a `cortile`

Riscrittura, non porting meccanico. Revisione upstream eseguita in analisi: branch `develop`, tree SHA `279c5cc5746814bbd8c339653302b01378d4ad3d`, ultima release v2.5.2 (2024-11-02). Esiti principali:

| Area | Direzione di Ktile |
|---|---|
| Linguaggio | Rust; niente runtime Go |
| Dipendenze | Set minimo, `Cargo.lock`, audit; backend unico `x11rb` |
| WM | Client X11 cooperante con Muffin |
| Geometria | Core puro e testabile, indipendente da X11 |
| Sicurezza operativa | Solo finestre client eleggibili; nessun `sudo`; niente rete, IPC o grab globali |
| Configurazione | TOML validato, default conservativi, chiavi uniche (D2: `outer`) |
| Diagnostica | Log strutturati e codici d'uscita stabili |
| Distribuzione | Binario utente in `~/.local/bin`, autostart facoltativo |

Funzionalità di Cortile esplicitamente non ereditate: systray, server D-Bus, hot corner, keybinding globali, addon Python, self-update e chiamate di rete (classificate SCARTATE in analisi).

## 4. Piattaforma e requisiti

Target iniziale:
- Linux Mint Cinnamon 22.x, sessione X11;
- Muffin come window manager (senza assumere supporto completo di ogni proprietà EWMH);
- Xorg con EWMH; x86_64 iniziale, ARM64 solo dopo test espliciti;
- Rust stable recente + Cargo;
- shell Bash/zsh per i comandi di sviluppo.

Ktile non dipende da `zsh`, `btop`, `eza` né da device specifici. Installazione utente in `~/.local/bin`, configurazione in `~/.config/ktile/`. Nessun daemon esterno obbligatorio, nessuna connessione di rete, nessun accesso root.

## 5. Direzione UX ispirata a Hyprland

Priorità confermate (D5):
- **Dwindle:** ogni nuova finestra divide una regione esistente in due (albero binario persistente); il verso segue `split_mode` deterministico (`longest-side` default, o `alternating`).
- **Master-stack:** una regione master e una colonna secondaria; `new_status` default `slave` (D7), promozione dello slave più vecchio a chiusura della master.
- **Gaps:** `inner` e `outer` configurabili, senza coprire pannelli.
- **Floating:** dialoghi, transient, popup flottanti/esclusi di default; esclusioni per applicazione configurabili.
- **Focus:** Ktile osserva `_NET_ACTIVE_WINDOW` ma non forza il focus né ruba input; il focus influenza solo la scelta della foglia da dividere in dwindle (`dwindle.split_target`, D3).
- **Righe di decisione:** regole per finestra/workspace dopo la verifica di workspace e monitor (M7).

Non replicabile sopra Muffin: animazioni, blur, bordi colorati, eventi pointer-aware di Hyprland. La v1 offre gaps e geometrie coerenti; l'indicatore di finestra attiva va valutato con le capacità di Muffin, senza overlay invasivi.

## 6. Principi di progettazione

1. **Muffin resta autorevole** per focus, stacking, decorazioni, workspace.
2. **Event-driven** con riconciliazione al bootstrap e dopo errori significativi.
3. **Idempotenza:** riapplicare un layout non produce spostamenti cumulativi.
4. **Conservatorismo:** proprietà EWMH ambigue/incomplete → non forzare il tiling.
5. **Geometria pura:** il motore di layout non apre connessioni X11.
6. **Recuperabilità:** SIGTERM, errori X11 e config invalida lasciano il controllo a Muffin.
7. **Determinismo:** stessa sequenza di eventi → stesso layout (D3).

## 7. Criteri di accettazione iniziali

- Il listener si avvia e termina correttamente in una sessione Cinnamon X11.
- Elenca le finestre client EWMH senza includere finestre interne del WM.
- Ignora finestre non eleggibili; pannelli e area di lavoro intatti.
- **Dwindle e master-stack** superano test deterministici senza connessione X11.
- Il programma non si blocca se una finestra scompare durante l'operazione.
- Nessun comando richiede `sudo`.
