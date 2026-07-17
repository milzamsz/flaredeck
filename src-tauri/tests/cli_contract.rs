use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    let config = std::env::temp_dir().join(format!("flaredeck-cli-{}", uuid::Uuid::new_v4()));
    Command::new(env!("CARGO_BIN_EXE_flaredeck-cli"))
        .env("XDG_CONFIG_HOME", config)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn doctor_json_uses_the_stable_envelope_and_stdout_only() {
    let output = run(&["--output", "json", "doctor"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["ok"], true);
    assert_eq!(value["meta"]["schemaVersion"], "1");
    assert!(value["meta"]["correlationId"]
        .as_str()
        .unwrap()
        .starts_with("corr_"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("\u{1b}["));
}

#[test]
fn usage_error_has_stable_code_exit_and_does_not_echo_canary() {
    let output = run(&["--output=json", "unknown", "CANARY_SECRET=must-not-escape"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["error"]["code"], "USAGE_ERROR");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("must-not-escape"));
}

#[test]
fn tunnel_status_is_observational_and_never_returns_process_details() {
    let output = run(&["--output=json", "tunnel", "status", "missing-profile"]);
    assert_eq!(output.status.code(), Some(10));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["error"]["code"], "VALIDATION_FAILED");
    assert!(!value.to_string().contains("pid"));
}
