use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "ktile",
    version = env!("CARGO_PKG_VERSION"),
    about = "Tiler X11 cooperante con Muffin/Cinnamon",
    long_about = None
)]
pub struct CliArgs {
    /// Valida il file di configurazione (ktile.toml) e ne stampa la struttura effettiva.
    #[arg(long)]
    pub check_config: bool,

    /// Elenca le finestre rilevate su X11 con relativi metadati ed eleggibilità.
    #[arg(long)]
    pub list_windows: bool,

    /// Stampa il piano di posizionamento calcolato senza inviare richieste al server X11.
    #[arg(long)]
    pub dry_run: bool,

    /// Seleziona il layout di tiling ("dwindle" o "master-stack").
    #[arg(short, long, value_parser = ["dwindle", "master-stack"])]
    pub layout: Option<String>,

    /// Specificatore percorso alternativo per il file di configurazione ktile.toml.
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Esegue Ktile in modalità daemon in foreground.
    #[arg(long)]
    pub daemon: bool,

    /// Consente l'applicazione reale delle geometrie (richiede M5 + consenso utente).
    #[arg(long)]
    pub allow_apply: bool,
}

impl CliArgs {
    /// Parsing dei comandi e flag da riga di comando.
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Ritorna true se la sessione X11 DISPLAY è disponibile.
    pub fn has_display(&self) -> bool {
        std::env::var_os("DISPLAY").is_some()
    }
}
