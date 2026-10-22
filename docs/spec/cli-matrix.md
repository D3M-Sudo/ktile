# CLI matrix — Ktile v1 (stato implementazione verificato al 2026-10-22)

Questa matrice descrive la CLI effettiva di `src/cli.rs` e `src/main.rs`; la presenza di un'opzione non implica che la relativa milestone sia chiusa sul target reale.

| Comando | Milestone | Connessione X11 | Effetto | Exit attesi |
|---|---|---|---|---|
| `ktile --version` | M1 | no | stampa versione | 0, 2 (flag invalido) |
| `ktile --check-config [--config PATH] [--layout dwindle\\|master-stack]` | M4 | no | valida TOML e stampa configurazione effettiva (l'override layout è accettato) | 0, 2, 3 (config invalida) |
| `ktile --list-windows` | M4 | sì (read-only) | elenca client EWMH: XID + class + desktop + type | 0 (anche zero eleggibili, D4), 4 (no DISPLAY), 5/6 (EWMH) |
| `ktile --dry-run --layout dwindle\|master-stack` | M4 | sì (read-only) | stampa piano XID→Rect, ZERO richieste | 0, 2, 3, 4, 5, 6 |
| `ktile --layout dwindle\|master-stack` | M5 | sì (write) | applica piano, max 3 richieste/finestra | 0, 2, 3, 4, 5, 6 |
| `ktile --daemon` | M6 (non implementato) | — | flag presente nel parser, ma event loop/daemon non implementato: non usarlo come funzione disponibile | da definire con M6 |

## Exit codes (D4, D6)

| Codice | Significato | Note |
|---|---|---|
| 0 | Successo, incluso "nessuna finestra eleggibile" (D4: log info, non errore) | amichevole per scripting |
| 2 | Argomenti CLI invalidi | clap |
| 3 | Configurazione invalida | validazione M4 |
| 4 | Sessione X11 non disponibile (DISPLAY assente/connessione fallita) | M1 |
| 5 | Errore X11/EWMH durante l'operazione | retry/riconciliazione poi exit |
| 6 | EWMH essenziale assente (no `_NET_SUPPORTING_WM_CHECK`/liste client) — NESSUNA whitelist nomi WM (D6) | parziali → warn + degradata, non exit |


## Config path: drift da risolvere

Le specifiche di progetto (`docs/spec/01-project-overview.md` e `README.md` storico) dichiarano `~/.config/ktile/config.toml`; il codice corrente in `src/main.rs` cerca invece `./ktile.toml` e poi `~/.config/ktile/ktile.toml`. Non assumere che uno dei due sia corretto senza una decisione esplicita: mantenere questo punto aperto fino alla scelta del contratto desiderato.
