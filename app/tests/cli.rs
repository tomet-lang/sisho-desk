//! Integration tests for `sisho-desk`.

use std::process::{Command, Stdio};

#[test]
fn without_a_terminal_it_says_so_and_exits_1() {
    let output = Command::new(env!("CARGO_BIN_EXE_sisho-desk"))
        .stdin(Stdio::null())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("sisho-desk: "), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    // Nothing was drawn: no escape sequences on standard output.
    assert!(output.stdout.is_empty(), "{:?}", output.stdout);
}

#[test]
fn the_help_describes_it() {
    let output = Command::new(env!("CARGO_BIN_EXE_sisho-desk"))
        .arg("--help")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("desk"), "{help}");
    assert!(help.contains("--config"), "{help}");
}
