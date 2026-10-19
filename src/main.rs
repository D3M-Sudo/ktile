// Entry point M1: wiring + exit codes.

use std::process::ExitCode;

use ktile::{cli, errors};

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
    // M1: connessione + verifica EWMH + elenco client + osservazione (read-only).
    Ok(0)
}
