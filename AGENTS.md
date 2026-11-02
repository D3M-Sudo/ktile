# Agent instructions

These instructions apply to automated coding agents and human contributors.

## Project scope
- Project: K-tile; repository: D3M-Sudo/ktile.
- Primary target: Linux Mint 22.3, Cinnamon/Muffin, X11.
- Language: Rust. Keep the core backend-independent.
- Priorities: low idle CPU/RAM, stability, explicit errors and small dependencies.
- Wayland is future work until a real compositor-specific backend is implemented and tested.
- Cortile is a behavior reference; reused source requires attribution and license preservation.

## Required workflow
1. Inspect branch, worktree, source and tests before editing.
2. Check existing issues and code paths before duplicating work.
3. Make coherent changes with regression tests.
4. Run formatting, tests, Clippy and release build when the environment supports them.
5. Report exact commands and outcomes; never claim unverified checks passed.
6. Do not turn failing quality gates into warnings or remove tests to make CI green.
7. Do not perform destructive desktop actions or enable live geometry changes without opt-in and a tested recovery path.
8. Do not install packages, add dependencies, delete branches, rewrite history, push, open PRs or merge without authorization.
9. Never commit secrets, tokens or private desktop content.
10. Do not mark roadmap items complete without evidence.

## Invariants
- Layout planning is deterministic and has no I/O.
- Never assume a fixed display size or zero-origin monitor coordinates.
- Exclude modal, fullscreen, maximized, transient, panel/dock and user-excluded windows by default unless a tested policy says otherwise.
- Revalidate window state immediately before every geometry mutation.
- Capture original geometry and do not overwrite later user changes on restore.
- Handle disappearing windows without crashing the daemon.
- Use event-driven reconciliation with bounded coalescing, not busy loops.
- Respect XDG paths and Muffin behavior.
- Keep GUI optional for headless tests.
- Preserve MIT and upstream notices.

## Reporting
Distinguish implementation, automated evidence, CI status and manual Cinnamon/Muffin checks. List blockers and the next safe step.
