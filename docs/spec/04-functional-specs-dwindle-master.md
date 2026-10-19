# 04 — Specifiche funzionali (layout core dwindle e master-stack)

Versione aggiornata alle decisioni chiuse il 2026-10-17 (D1–D9). I valori qui indicati sono definitivi salvo nuova decisione registrata.

## 1. Comandi CLI

```text
ktile --version
ktile --check-config
ktile --list-windows
ktile --dry-run --layout dwindle|master-stack
ktile --layout dwindle|master-stack
ktile --daemon
```

- `--layout` ha default da config; `--dry-run` stampa il piano senza applicarlo.
- `--daemon` è foreground e opt-in; niente PID file, niente fork; shutdown su SIGTERM/SIGINT entro 1 s.
- `--check-config` valida e stampa la config effettiva; nessuna connessione richiesta.

## 2. Configurazione TOML

Esempio con i default:

```toml
[general]
layout = "dwindle"            # "dwindle" | "master-stack"
follow_monitor_focus = false  # multi-monitor: futuro (M7)

[gaps]
inner = 8                     # gap tra le finestre
outer = 8                     # gap dai bordi dell'area di lavoro (D2: unico nome)

[dwindle]
split_mode = "longest-side"   # "longest-side" | "alternating"
split_ratio = 0.5             # clamp runtime [0.25, 0.75]
split_target = "active"       # "active" | "mru" (D3: foglia da dividere)

[master]
mfact = 0.55                  # clamp [0.25, 0.75]
new_status = "slave"          # "slave" | "master" (D7)

[behavior]
new_is_master = false         # alias legacy di new_status = "master" (warn se entrambi)
single_monitor_only = true    # v1: errore chiaro se più monitor (M7 rimuove)
smart_gaps = false
respect_monitor_reserved = true

[exclude]
classes = ["kupfer.py", "desktop_window"]
instances = ["conky"]
title_regex = ["^np\.vc\.virtualbox$"]
```

Regole di validazione (`--check-config`): gap non negativi e < metà dell'area minima; enum chiusi; regex compilabili; chiavi sconosciute → warning o errore secondo policy (default: errore). Default conservativi; config assente → creazione con commenti.

## 3. Eleggibilità delle finestre

Eleggibile solo se **tutte** le condizioni (AND):

1. Presente in `_NET_CLIENT_LIST` (o stacking in fallback);
2. `_NET_WM_WINDOW_TYPE` assente o NORMAL (override `include_types` non previsto in v1);
3. Non fullscreen (FULLSCREEN) — mai toccata;
4. Non transient (`WM_TRANSIENT_FOR` assente) — esclusa di default, override per-app possibile;
5. Tipo diverso da DOCK, DESKTOP, TOOLBAR, MENU, TOOLTIP, SPLASH, NOTIFICATION, DROPDOWN/POPUP MENU, DND, COMBO, UTILITY (UTILITY borderline: esclusa in v1);
6. Stato diverso da HIDDEN, SKIP_TASKBAR, SKIP_PAGER, MODAL, ABOVE/BELOW;
7. Workspace attivo (`_NET_WM_DESKTOP` = corrente); 0xFFFFFFFF = sticky → **esclusa in v1** (D9);
8. Non esclusa da regole config (class/instance primari, titolo solo secondario);
9. Mappata e visibile.

Stati della registry: `Active` (mappata, nel layout) → `Suspended` (unmap/HIDDEN: slot rilasciato, posizione conservata, mai toccata) → ripristino solo da eventi; `Excluded` (presente, mai nel layout); `Gone` (DestroyNotify: rimozione definitiva).

| Categoria | Policy v1 |
|---|---|
| Dialoghi e transient | Esclusi (flottanti) di default; override per-app |
| Popup, menu, tooltip | Esclusi, mai toccati |
| Dock e pannelli | Mai toccati; usati solo per l'area (strut) |
| Fullscreen | Esclusi; nessuna geometria; mai modificati |
| Maximized | Esclusi finché maximized; nessun unmaximize automatico in v1 |
| Minimizzate | Sospese (slot rilasciato); mai ripristinate da Ktile |
| Sticky | Escluse in v1 (D9), rivalutazione in M7 |

Ripristino di una sospesa (D8): rientro nella **posizione originaria** se lo slot è ancora identificabile; altrimenti inserimento come nuova foglia (regola standard).

## 4. Layout dwindle

- **Struttura:** albero binario persistente per ogni `(desktop, monitor)`. Split persistente (≈ `preserve_split=true` di Hyprland): direzione e rapporto restano memorizzati; la stessa sequenza di eventi produce sempre lo stesso albero.
- **Inserimento:** la nuova finestra occupa `second`; il contenuto preesistente della foglia target resta in `first`. Foglia target = foglia della finestra attiva se tiled (`split_target = "active"`), altrimenti la foglia MRU (più di recente usata). Workspace vuoto → la finestra riempie l'area meno i margini.
- **Direzione split** (`split_mode`): `longest-side` (default: orizzontale se la regione è più larga che alta, verticale altrimenti); `alternating` alterna rispetto al genitore. Niente dipendenza dal puntatore (non replicabile in modo affidabile sopra Muffin).
- **Rapporto iniziale:** `split_ratio = 0.5` (milli interni 500), clamp [0.25, 0.75].
- **Gaps:** `outer` una volta sull'area; `inner` tra le due metà di ogni split. Residui ≤1px: parte intera a `first` (convenzione testata). Area < gap → clamp definito + log, mai panic.
- **Rimozione:** chiusura/distruzione → il padre del `Leaf` è sostituito dal sottoalbero fratello (promozione). Nessun riequilibrio: gli altri rami non cambiano.
- **Finestra attiva floating/esclusa:** non occupa slot; la selezione foglia ricade sull'MRU tiled.
- **Minimizzate/nascoste:** uscita dall'albero come rimozione (promozione fratello); rientro secondo D8.
- **Cambio workspace/monitor:** ogni workspace ha il proprio albero; spostamento = rimozione sorgente + inserimento destinazione; ricalcolo solo se lo stato X11 differisce dall'aspettativa (idempotenza).
- **Limiti dimensionali:** se una metà scenderebbe sotto `MIN_SIDE` (default 50px) o sotto il min ICCCM letto, la nuova finestra va nel **ramo esistente più capiente** (stesso criterio longest-side, fallback MRU); se nessun ramo la accoglie → resta flottante (log). Mai richieste illegali.
- **Focus:** Ktile non forza il focus; il focus influisce solo su `split_target`.

## 5. Layout master-stack

- **Selezione master:** prima finestra eleggibile del workspace. `new_status = "slave"` (default, D7): la nuova finestra entra in testa o in coda allo stack secondo `new_on_top` (futuro; v1: in coda, ordine stabile D3). Con `new_status = "master"` la nuova finestra diventa master e la master corrente scivola nello stack.
- **Disposizione slave:** colonna secondaria accanto alla master; slave impilati verticalmente a ripartizione uguale con proporzioni persistenti (come dwindle). Tutti gli slave hanno uno slot visibile (niente stacking nascosto).
- **Promozione:** master chiusa → lo **slave più vecchio** diventa master (anche con `new_status = "master"`).
- **Proporzioni:** `mfact = 0.55`, clamp [0.25, 0.75]; modifica solo via config/CLI in v1 (resize interattivo rimandato).
- **Relazione col focus:** nessun focus forzato.
- **Differenze vs dwindle:** un solo punto decisionale (mfact) vs un albero; chiusura master = riassegnazione di ruolo vs riassorbimento di regione; con 1 finestra entrambi coincidono (area meno outer).

## 6. Applicazione delle geometrie

Gate per ogni richiesta: finestra esistente (BadWindow → WindowGone), eleggibile, non fullscreen, dimensioni positive, rispetto minimi ICCCM (sola lettura). Preferire i meccanismi che la verifica M5 dimostrerà efficaci su Muffin. Dopo la richiesta: leggere il risultato reale; se "vicino" (tolleranza 1–2 px) → aggiornare stato atteso; se lontano → un solo rinvio; poi deviazione loggata in diagnostica. Mai riscrivere hint ICCCM; su Muffin/Mutter mai `WM_NORMAL_HINTS` (vincolo analisi Cortile, da re-verificare).

## 7. Anti-feedback e race

1. **Geometry in flight:** (XID, Rect, timestamp) registrato dall'applier; i ConfigureNotify coincidenti entro tolleranza vengono consumati.
2. **Trigger selettivi:** ricalcolo solo da Map/Unmap/Destroy/PropertyNotify rilevanti, mai da ogni ConfigureNotify.
3. **Piano idempotente:** piano identico allo stato atteso → nessuna richiesta.
4. **Anti-livelock:** max 3 ricalcoli per burst poi coalescing con timer monotònico (mai polling continuo).

Race su chiusura durante richiesta: nessun retry cieco; riconciliazione completa solo dopo errori ripetuti; mai panic.

## 8. Codici d'uscita

| Codice | Significato |
|---|---|
| 0 | Successo, incluso "nessuna finestra eleggibile" (D4: log informativo) |
| 2 | Argomenti CLI invalidi o help richiesto con errore |
| 3 | Configurazione invalida |
| 4 | Sessione X11 non disponibile (DISPLAY assente o connessione fallita) |
| 5 | Errore X11/EWMH durante l'operazione |
| 6 | EWMH essenziale assente: nessun WM EWMH rilevato (`_NET_SUPPORTING_WM_CHECK`/liste client) — **nessuna whitelist di nomi WM** (D6); proprietà EWMH parziali → warn + modalità degradata |

Log su stderr con livelli (error/warn/info/debug); opzione verbosity; nessun log file obbligatorio.

## 9. Ciclo di vita

- Bootstrap: connessione → sanity check EWMH → snapshot stato → calcolo piano → (dry-run o applicazione).
- Event loop: Map/Unmap/Destroy/PropertyNotify/RandR → ricalcolo selettivo.
- Errori: singolo errore X11 → warn; errori ripetuti → riconciliazione; connessione persa → exit 5.
- SIGTERM/SIGINT: stop pulito entro 1 s; nessuna richiesta pendente ri-inviata; Ktile non ripristina nulla all'uscita.

## 10. Stato non funzionale

- V1: layout globale (stessa scelta per tutti i workspace); per-workspace/per-monitor in M7.
- V1: niente cache di geometrie su disco; niente persistenza dello stato tra riavvii (valutazione futura).
- V1: niente overlay indicatori; gap e geometrie sono l'indicazione primaria.
- V1: nessuna interazione con il puntatore (drag swap/resize rimandati).

## 11. Note sui nomi delle chiavi (stabilizzazione post-decisioni)

- D2: `outer` è l'unica chiave per il gap esterno; `gaps_out` non è riconosciuta.
- D3: `focus_first` non esiste più; uso di `dwindle.split_target`.
- D7: `new_is_master = true` equivale a `new_status = "master"` (se entrambi presenti e in conflitto → errore di validazione).
- Alias legacy generici: un solo alias per chiave, documentato, con warning di deprecazione.
