# 03 — Compatibilità EWMH/Muffin (ricerca e protocollo di verifica)

Ktile è un client X11 cooperante con Muffin: Muffin resta autorevole su focus, stacking, decorazioni e workspace. Compatibilità significa osservare correttamente le proprietà EWMH/ICCCM, applicare geometrie richieste senza rubare il controllo e gestire mancanze o comportamenti anomali senza danneggiare la sessione. Muffin è un fork di Mutter; alcuni comportamenti Mutter-like (posizionamento proprio di finestre nuove, ricalcolo di hint, gestione prudente di alcune richieste) sono attesi e vanno verificati sul target.

## 1. Proprietà chiave e strategia

| Proprietà/evento | Uso in Ktile | Note su Muffin (da verificare sul target) |
|---|---|---|
| `_NET_SUPPORTING_WM_CHECK` | Verifica EWMH: lettura root → XID, rilettura sulla finestra WM; nome da `_NET_WM_NAME` | Se assente → exit 6 (D6) |
| `_NET_SUPPORTED` | Elenco atomi supportati; guida la modalità degradata | Proprietà parziali → warn + feature ridotte, non rifiuto |
| `_NET_CLIENT_LIST`, `_NET_CLIENT_LIST_STACKING` | Registro finestre client; stacking come ordine stabile (D3) | Assenti → fallback all'altro; entrambi assenti → exit 6 |
| `_NET_ACTIVE_WINDOW` | Indicazione focus (fonte primaria); rileggere su PropertyNotify | Indicazione, non focus reale; ritardi possibili |
| `_NET_WM_DESKTOP` | Appartenenza al workspace; 0xFFFFFFFF = sticky | Sticky escluse in v1 (D9) |
| `_NET_WM_STATE` | FULLSCREEN/MAXIMIZED/HIDDEN/SKIP_*/MODAL/ABOVE/BELOW | Cambiamenti durante il tiling: rileggere |
| `_NET_WM_WINDOW_TYPE` | Eleggibilità (default NORMAL se assente) | App senza tipo → trattare come NORMAL |
| `_NET_WORKAREA` | Controllo incrociato aree, trigger rebuild | **Non** fonte primaria: per-desktop, approssimata su multi-monitor |
| `_NET_WM_STRUT_PARTIAL` (+`_NET_WM_STRUT`) | Area pannello/dock; fonte primaria per l'area utile (con RandR) | STRUT non-partial a granularità schermo intero |
| `_NET_FRAME_EXTENTS` | Estensione cornice SSD; combinare con `_GTK_FRAME_EXTENTS` (somma NET, sottrai GTK) | Possibile assenza/ritardo post-map; CSD: documentare discrepanze residue |
| `WM_CLASS`, `WM_NAME` | Identificazione app per regole/esclusioni | Fallback su titolo |
| `WM_TRANSIENT_FOR` | Dialoghi/transient → esclusi di default | — |
| `WM_NORMAL_HINTS` | **Sola lettura**: minimi dimensionali per validare le geometrie richieste | **Su Muffin/Mutter non riscrivere mai queste hint** (vincolo emerso dall'analisi di Cortile, `Compatible("icccm.SizeHintPMinSize")`) — da re-verificare |
| Map/Unmap/Destroy | Ciclo di vita: Active → Suspended (D8) → Gone | Unmap ≠ destroy (minimize, cambio workspace) |
| ConfigureNotify | Geometria effettiva dopo richiesta; filtro anti-feedback | Muffin può modificare/ignorare richieste |
| PropertyNotify | Cambi proprietà; rileggere sempre il valore | — |

## 2. Categorie EWMH (sintesi)

- **Identificazione WM**: `_NET_SUPPORTING_WM_CHECK`, `_NET_WM_NAME`.
- **Desktop/virtual**: `_NET_NUMBER_OF_DESKTOPS`, `_NET_CURRENT_DESKTOP`, `_NET_DESKTOP_NAMES`, `_NET_DESKTOP_GEOMETRY`, `_NET_DESKTOP_VIEWPORT`, `_NET_WM_DESKTOP`.
- **Client list/stacking**: `_NET_CLIENT_LIST`, `_NET_CLIENT_LIST_STACKING`.
- **Window states/types**: `_NET_WM_STATE`, `_NET_WM_STATE_*`, `_NET_WM_WINDOW_TYPE`, `_NET_WM_WINDOW_TYPE_*`.
- **Area di lavoro**: `_NET_WORKAREA`, `_NET_WM_STRUT`, `_NET_WM_STRUT_PARTIAL`.
- **Decorazioni/frame**: `_NET_FRAME_EXTENTS`, `_NET_REQUEST_FRAME_EXTENTS`, `_GTK_FRAME_EXTENTS` (de-facto GTK), Motif hints (lettura).
- **Focus**: `_NET_ACTIVE_WINDOW`, FocusIn/FocusOut.
- **Fullscreen**: `_NET_WM_STATE_FULLSCREEN` (segnale per non toccare la finestra).

## 3. Pannelli Cinnamon

1. Geometrie monitor via **RandR** (`x11rb` ext `randr`); v1: un solo monitor con messaggio chiaro se più di uno.
2. Per ogni finestra DOCK della client list: `_NET_WM_STRUT_PARTIAL` → rettangoli riservati; area utile = monitor ∩ complemento strut.
3. Controllo incrociato con `_NET_WORKAREA` (diagnostica).
4. Casi: pannello alto/basso/laterale, auto-hide, secondo pannello.

**Vietato** compensare con offset fissi ("sottrai 32px in alto"): i pannelli sono configurabili e spostabili.

## 4. Monitor multipli

V1: singolo monitor. I tipi `Monitor`/`WorkspaceArea` sono già multi-monitor-ready (WorkspaceId = (desktop, monitor)) così M7 non richiede rifattorizzazione. Hotplug: rebuild completo a fronte di eventi RandR o PropertyNotify su `_NET_DESKTOP_GEOMETRY`; mai aggiornamenti parziali.

## 5. Decorazioni SSD e CSD

- SSD: Muffin disegna la cornice; usare `_NET_FRAME_EXTENTS` quando presente.
- CSD (headerbar GTK): `_GTK_FRAME_EXTENTS` descrive l'ingombro; combinarlo con NET (approccio dell'analisi su Cortile, `GetInfo()`).
- Policy v1: **non modificare decorazioni**; le discrepanze CSD residue sono limiti documentati, non da "risolvere" con euristiche aggressive.

## 6. Regole di sicurezza su finestre ed eventi

- Non toccare mai finestra fullscreen/maximized: niente geometrie.
- Minimized: fuori dallo slot (sospesa, D8); mai ripristinarla da Ktile.
- Unmap senza cambio di proprietà: non assumere chiusura; DestroyNotify è l'unica rimozione definitiva.
- Se una finestra viene ridimensionata/spostata mentre una richiesta è in corso: usare la geometria più recente e non riaccodare richieste obsolete.
- Ignorare eventi propri di Ktile per il filtro anti-feedback (geometry in flight + tolleranza 1–2 px).
- `BadWindow` asincrono → evento di dominio `WindowGone`, mai panic; riconciliazione dopo errori ripetuti.
- Mai usare grab globali di tastiera o server.

## 7. Gestione assenze e comportamenti anomali

- Se una proprietà EWMH attesa è assente: fallback definito nella tabella sopra; altrimenti warn e skip della feature (D6).
- Se il WM non supporta `_NET_MOVERESIZE_WINDOW` o messaggi client: testare `ConfigureWindow`; registrare quale percorso funziona su Muffin (M5).
- Se il WM forza posizioni: accettare il risultato, loggare, non insistere (max 3 tentativi per burst).

## 8. Test di compatibilità su Linux Mint Cinnamon (target)

Strumenti: `xprop`, `xwininfo`, `xev`, `xrandr` (se assenti: non installare, registrare e chiedere).

```bash
xprop -root _NET_SUPPORTING_WM_CHECK _NET_SUPPORTED _NET_WORKAREA \
      _NET_CLIENT_LIST _NET_CLIENT_LIST_STACKING _NET_ACTIVE_WINDOW \
      _NET_NUMBER_OF_DESKTOPS _NET_CURRENT_DESKTOP _NET_DESKTOP_GEOMETRY
# per finestra normale, dialogo/transient e pannello:
xprop -id 0xXXXX _NET_WM_STATE _NET_WM_WINDOW_TYPE _NET_WM_DESKTOP \
      _NET_FRAME_EXTENTS _GTK_FRAME_EXTENTS _NET_WM_STRUT_PARTIAL \
      WM_TRANSIENT_FOR WM_NORMAL_HINTS WM_CLASS
xrandr --listmonitors
```

Casi manuali: pannello alto/basso/laterale/auto-hide; dialogo GTK e Qt; app Electron; finestra CSD vs SSD; minimize/restore; spostamento tra workspace; fullscreen; chiusura durante operazioni; app che resistono alle dimensioni.

Output in `docs/milestones/environment-baseline.md` (M0) e `docs/milestones/M1-findings.md` (M1), con marcatura "verificato/ipotesi".

## 9. Ipotesi non confermate (elenco da chiudere con i test)

1. Muffin applica esattamente le geometrie richieste per finestre tiled (o modifica/ignora).
2. Comportamento di `_NET_WORKAREA` su Muffin recente, anche con pannello laterale e auto-hide.
3. Supporto di messaggi client `_NET_WM_STATE` per (un)maximize e la risposta di Muffin.
4. Trattamento di `_GTK_FRAME_EXTENTS` per app CSD su GTK recente.
5. Sequenza effettiva di MapNotify/UnmapNotify per minimize/cambio workspace (Unmap "falso" vs destroy).
6. Aggiornamento dei `_NET_WM_STRUT_PARTIAL` in tempo reale con auto-hide del pannello.
7. Numero di workspace e viewport Cinnamon di default; eventi di notifica adeguati.
8. Nome WM riportato da `_NET_WM_NAME` ("Muffin" vs altro) e completezza di `_NET_SUPPORTED`.
9. Effettiva stabilità del focus in `_NET_ACTIVE_WINDOW` durante alt-tab/switcher.
10. Effetto dell'impostazione Cinnamon "edge tiling" sulle finestre tiled da Ktile (interazione).

Ogni ipotesi chiusa va registrata con data, versione Cinnamon/Muffin e comando usato.
