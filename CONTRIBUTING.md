# Contributing

Read README.md, AGENTS.md, docs/architecture/ARCHITECTURE.md, docs/planning/ROADMAP.md and SECURITY.md before substantial changes. Search existing issues and pull requests first. Discuss major behavior changes before implementing them.

## Development checks
Install stable Rust with Cargo, rustfmt and Clippy. Run:

    make check

Do not weaken CI gates to hide failures. Report checks that could not run.

## Engineering expectations
- Keep the layout/domain core independent from X11, GTK, IPC and filesystem I/O.
- Avoid hard-coded screen dimensions and zero-origin assumptions.
- Prefer event-driven reconciliation over polling.
- Revalidate eligibility immediately before geometry changes.
- Add regression tests and document behavior changes and limitations.
- Explain new runtime dependencies and review their licenses.
- Include manual Cinnamon/Muffin checks where relevant.

## Pull requests
Include rationale, implementation summary, exact tests and results, remaining manual checks, risks and rollback. Update documentation. Preserve required upstream attribution for copied/adapted code. Do not include secrets, private logs or generated build artifacts.

By submitting a contribution, you agree it may be distributed under the repository's MIT License. You retain copyright in your contribution. See docs/development/CORTILE-PORTING.md for third-party attribution requirements.
