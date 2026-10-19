// DRAFT M1-prep — NON compilato, NON testato: toolchain Rust assente sull'host.
// Gate cargo (fmt/clippy/test) sospeso. Entry point M1: wiring + exit codes.

mod cli;
mod errors;
mod x11;
mod state;

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("ktile: {e}");
            ExitCode::from(e.exit_code())
        }
    }
}

fn run() -> Result<u8, errors::KtileError> {
    let args = cli::parse();
    if !args.has_display() {
        return Err(errors::KtileError::NoDisplay);
    }
    // M1: connessione + verifica EWMH + elenco client + osservazione 30s (read-only).
    Ok(0)
}
