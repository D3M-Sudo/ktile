// Test exit codes (doc 04 §8): errori → codici stabili.

use ktile::errors::KtileError;

#[test]
fn no_display_is_exit_4() {
    assert_eq!(KtileError::NoDisplay.exit_code(), 4);
}

#[test]
fn missing_ewmh_is_exit_6() {
    assert_eq!(KtileError::EwmhMissing("test".to_string()).exit_code(), 6);
}

#[test]
fn invalid_args_is_exit_2() {
    assert_eq!(KtileError::Args("test".to_string()).exit_code(), 2);
}

#[test]
fn x11_error_is_exit_5() {
    assert_eq!(KtileError::X11("test".to_string()).exit_code(), 5);
}
