// DRAFT M1-prep — NON compilato, NON testato: toolchain Rust assente sull'host.
// CLI M1 (minima): --version. Estensioni M4: --check-config --list-windows --dry-run --layout --daemon.

/// Argomenti parsati (parser reale via clap in M1 con toolchain).
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// Richiesta versione.
    pub version: bool,
}

/// Parse minimale DRAFT (sostituire con clap::Parser in M1).
pub fn parse() -> Args {
    let mut args = Args::default();
    for a in std::env::args().skip(1) {
        if a == "--version" || a == "-V" {
            args.version = true;
        }
    }
    args
}

impl Args {
    /// True se DISPLAY è disponibile (exit 4 se assente — D6/D4).
    pub fn has_display(&self) -> bool {
        std::env::var_os("DISPLAY").is_some()
    }
}
