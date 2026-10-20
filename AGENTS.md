# AGENTS — Ktile: regole operative per agenti AI

> Letto per primo a ogni nuova sessione. Dettagli in `docs/handover/HANDOVER.md`,
> fonte di verità in `docs/ARCHITECTURE.md` + `docs/ROADMAP.md` (ADR-13: aggiornarli contestualmente a ogni modifica).

1. **Rust prima in CI (ADR-16).** Build, fmt, clippy e test girano in GitHub Actions
   (`.github/workflows/ci.yml`): verificare con `gh pr checks <n>` / `gh run list`
   e attendere i check verdi prima del merge. `cargo` su host solo come fallback
   se toolchain user-level già installata/autorizzata (amendment ADR-15);
   mai proporre installazioni di sistema.
2. **Host pulito (ADR-14).** Vietati `sudo`/`apt install`; tool Python via `uvx`
   one-shot o `.venv`. Su host consentiti solo `git`, `gh`, `grep`, letture docs.
3. **Zero scritture geometriche** fuori `x11/applier.rs` (gate: `grep -rniE 'configure_window|moveresize' src/ tests/` = 0).
4. **Muffin autorevole** su focus/stacking/decorazioni; mai riscrivere `WM_NORMAL_HINTS` (ADR-11).
5. **Test X11 in CI** (Xvfb+openbox) = "verificato su WM di test, da confermare su Muffin".
