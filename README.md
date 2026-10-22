# Ktile

Ktile è un tiler dinamico per **Linux Mint Cinnamon in sessione X11**, scritto in Rust e progettato per cooperare con Muffin anziché sostituirlo. Offre due layout: **Dwindle** (suddivisioni binarie persistenti) e **Master-Stack**, con geometria deterministica e gap configurabili.

> **Stato: prototipo in sviluppo.** M0–M5 sono implementate a livello di milestone di codice, con i gate target ancora aperti dove indicato; M3 ha superato i test e la soglia di copertura in CI (89,66% delle linee di `src/layout/`). M5 supporta l'applicazione manuale opt-in, ma il gate di accettazione su Cinnamon/Muffin reale è ancora aperto. Il daemon automatico (M6), la compatibilità avanzata (M7) e il packaging (M8) non sono ancora disponibili.

La verifica M3 è documentata nella [CI run #39843748187](https://github.com/D3M-Sudo/ktile/actions/runs/39843748187): 650/725 linee coperte (89,66%; soglia 80%). Questa verifica automatizzata non equivale all'accettazione su Cinnamon/Muffin reale.

## Principi

- Muffin rimane responsabile di focus, stacking, decorazioni e workspace.
- Ktile non è un compositor e non implementa blur, animazioni o bordi tramite hack grafici.
- Il comportamento predefinito e `--dry-run` sono read-only. Per inviare richieste geometriche è necessario `--allow-apply`.
- Nessun privilegio root e nessuna installazione di sistema richiesta per il progetto.

## Requisiti

- Linux con sessione X11 e un window manager EWMH; target principale: Linux Mint Cinnamon/Muffin.
- Rust stable e Cargo per compilare dai sorgenti.
- Per i test d'integrazione X11: Xvfb, Openbox e dipendenze di sviluppo X11 indicate in [CI](.github/workflows/ci.yml).

## Compilazione e uso da sorgente

Dalla directory del repository:

```bash
cargo build
cargo test
cargo run -- --check-config
cargo run -- --list-windows
cargo run -- --dry-run --layout dwindle
```

Per applicare geometrie, solo dopo aver controllato il piano e aver predisposto una finestra di prova:

```bash
cargo run -- --allow-apply --layout dwindle
```

**Attenzione:** `--allow-apply` può spostare o ridimensionare le finestre eleggibili della sessione corrente. I test CI con Xvfb/Openbox non certificano il comportamento su Muffin. Prima di provarlo su un desktop reale, leggere la [procedura locale M5](docs/ROADMAP.md#procedura-operativa-m5-per-il-test-locale).

Non è ancora disponibile una procedura di installazione/release stabile: il packaging utente è pianificato per M8.

## Configurazione: contratto ancora da decidere

Il percorso storico `~/.config/ktile/config.toml` diverge dal comportamento attuale del codice, che cerca `./ktile.toml` e poi `~/.config/ktile/ktile.toml`; senza file usa i default in memoria. La decisione è deliberatamente aperta: **non assumere che uno dei percorsi sia canonico**. Vedi [panoramica e nota di divergenza](docs/spec/01-project-overview.md#drift-da-risolvere-percorso-canonico-della-configurazione).

## Documentazione per sviluppatori

- [Roadmap e checklist di accettazione](docs/ROADMAP.md)
- [Architettura e stato dei moduli](docs/ARCHITECTURE.md)
- [Passaggio di consegne per agenti](docs/handover/HANDOVER.md)
- [Panoramica e obiettivi](docs/spec/01-project-overview.md)
- [Architettura Rust/x11rb](docs/spec/02-architecture-rust-x11rb.md)
- [Specifiche funzionali e CLI](docs/spec/04-functional-specs-dwindle-master.md)
- [Compatibilità EWMH/Muffin e protocollo di verifica](docs/research/03-ewmh-muffin-compatibility.md)

Le verifiche manuali ancora aperte, inclusi i test M4 su pannelli/workarea e i test M5 su Cinnamon/Muffin, sono elencate in `docs/ROADMAP.md` e devono restare aperte finché non vengono eseguite sul target.

## Contribuire

- Leggere prima `AGENTS.md`, poi `docs/handover/HANDOVER.md`.
- Mantenere `docs/ARCHITECTURE.md` e `docs/ROADMAP.md` coerenti con codice e test.
- Eseguire fmt, Clippy, build, test e controlli coverage in GitHub Actions; non installare tool di sistema sull'host.
- Non creare PR né fare merge senza esplicita autorizzazione del proprietario del repository.
