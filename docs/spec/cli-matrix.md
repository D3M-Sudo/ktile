# CLI matrix — Ktile v1 (M1-prep, DRAFT: comandi da implementare in M1/M4)

| Comando | Milestone | Connessione X11 | Effetto | Exit attesi |
|---|---|---|---|---|
| `ktile --version` | M1 | no | stampa versione | 0, 2 (flag invalido) |
| `ktile --check-config [--config PATH]` | M4 | no | valida TOML, stampa config effettiva | 0, 2, 3 (config invalida) |
| `ktile --list-windows` | M4 | sì (read-only) | elenca client EWMH: XID + class + desktop + type | 0 (anche zero eleggibili, D4), 4 (no DISPLAY), 5/6 (EWMH) |
| `ktile --dry-run --layout dwindle\|master-stack` | M4 | sì (read-only) | stampa piano XID→Rect, ZERO richieste | 0, 2, 3, 4, 5, 6 |
| `ktile --layout dwindle\|master-stack` | M5 | sì (write) | applica piano, max 3 richieste/finestra | 0, 2, 3, 4, 5, 6 |
| `ktile --daemon` | M6 | sì (write) | foreground opt-in, shutdown ≤1s su SIGTERM/SIGINT | 0, 2, 3, 4, 5, 6 |

## Exit codes (D4, D6)

| Codice | Significato | Note |
|---|---|---|
| 0 | Successo, incluso "nessuna finestra eleggibile" (D4: log info, non errore) | amichevole per scripting |
| 2 | Argomenti CLI invalidi | clap |
| 3 | Configurazione invalida | validazione M4 |
| 4 | Sessione X11 non disponibile (DISPLAY assente/connessione fallita) | M1 |
| 5 | Errore X11/EWMH durante l'operazione | retry/riconciliazione poi exit |
| 6 | EWMH essenziale assente (no `_NET_SUPPORTING_WM_CHECK`/liste client) — NESSUNA whitelist nomi WM (D6) | parziali → warn + degradata, non exit |
