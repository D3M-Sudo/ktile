// DRAFT M1-prep — NON compilato, NON testato: toolchain Rust assente sull'host.
// Test exit codes (doc 04 §8): errori → codici stabili. Esecuzione reale in M1 con toolchain.

#[test]
fn no_display_is_exit_4() {
    // DRAFT: KtileError::NoDisplay.exit_code() == 4
}

#[test]
fn missing_ewmh_is_exit_6() {
    // DRAFT: KtileError::EwmhMissing(_).exit_code() == 6
}

#[test]
fn invalid_args_is_exit_2() {
    // DRAFT: KtileError::Args(_).exit_code() == 2
}
