# AGENTS — Ktile: regole operative per agenti AI

Leggere questo file all'inizio di ogni sessione, poi `docs/handover/HANDOVER.md`. Le fonti di verità per stato e architettura sono `docs/ROADMAP.md` e `docs/ARCHITECTURE.md`.

1. **Nessuna PR o merge senza autorizzazione esplicita.** Lavorare sul ramo richiesto; prima di scrivere verificare branch e HEAD correnti.
2. **CI prima scelta (ADR-16).** Verificare GitHub Actions per fmt, Clippy, build, test e copertura. Per M3, `src/layout/` deve mantenere almeno l'80% di line coverage secondo il job `cargo llvm-cov`. Non dichiarare verde una run in corso o non consultata. Toolchain host solo se già installata e autorizzata; mai proporre `sudo`/installazioni di sistema.
3. **Host conservativo (ADR-14/15).** Non installare pacchetti di sistema. Per tool Python usare `uvx` o una `.venv`; rispettare le autorizzazioni esplicite dell'utente.
4. **Confine delle scritture geometriche.** `src/x11/applier.rs` è l'unico modulo autorizzato a inviare richieste di modifica geometrica. Ogni match di `configure_window` o `moveresize` in `src/` e `tests/` va esaminato: non usare un gate che pretende zero match globali, perché il codice dell'applier e i test possono legittimamente contenerli.
5. **Muffin autorevole.** Non forzare focus, stacking o decorazioni e non riscrivere `WM_NORMAL_HINTS` (ADR-11).
6. **Distinguere CI e target reale.** Xvfb/Openbox verifica il WM di test, non certifica Muffin. Le checklist Cinnamon/X11 in `docs/ROADMAP.md` restano aperte finché l'utente non comunica risultati reali.
7. **Documentazione contestuale (ADR-13).** Aggiornare roadmap e architettura quando cambia lo stato effettivo; non trasformare ipotesi in risultati verificati.
8. **Contratto config non risolto.** Non cambiare percorso o comportamento della configurazione finché l'utente non decide quale percorso rendere canonico.
