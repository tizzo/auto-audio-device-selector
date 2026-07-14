//! Integration tests for the `--json` output mode of the binary.
//!
//! These exercise the read-only commands the Raycast extension depends on and,
//! importantly, guard against logging leaking onto stdout (which would corrupt
//! `JSON.parse` on the consumer side). Only read-only commands are used so the
//! tests never mutate the machine's audio device selection.

use serde_json::Value;
use std::process::Command;

/// Path to the compiled binary, provided by Cargo for integration tests.
const BIN: &str = env!("CARGO_BIN_EXE_audio-device-monitor");

/// Run the binary with the given args and return its stdout as a string,
/// asserting a successful exit status.
fn run(args: &[&str]) -> String {
    let output = Command::new(BIN)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {BIN}: {e}"));

    assert!(
        output.status.success(),
        "command {args:?} exited with {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );

    String::from_utf8(output.stdout).expect("stdout was not valid UTF-8")
}

/// stdout must be pure JSON — nothing before the opening brace. This is the
/// regression guard for logs being written to stdout in `--json` mode.
fn parse_clean_json(stdout: &str) -> Value {
    let trimmed = stdout.trim_start();
    assert!(
        trimmed.starts_with('{'),
        "stdout was not clean JSON (log lines leaked onto stdout?):\n{stdout}"
    );
    serde_json::from_str(trimmed).expect("stdout was not parseable JSON")
}

#[test]
fn list_devices_json_has_expected_shape() {
    let json = parse_clean_json(&run(&["list-devices", "--json"]));

    assert!(json.get("devices").is_some(), "missing `devices` key");
    assert!(json["devices"].is_array(), "`devices` should be an array");
    // Keys are always present, even when null.
    assert!(json.as_object().unwrap().contains_key("default_output"));
    assert!(json.as_object().unwrap().contains_key("default_input"));

    // Each device entry must expose the fields the extension relies on.
    for device in json["devices"].as_array().unwrap() {
        for key in ["id", "name", "type", "is_default", "is_available", "uid"] {
            assert!(
                device.as_object().unwrap().contains_key(key),
                "device entry missing `{key}`: {device}"
            );
        }
        let device_type = device["type"].as_str().unwrap();
        assert!(
            matches!(device_type, "Input" | "Output" | "Input/Output"),
            "unexpected device type: {device_type}"
        );
    }
}

#[test]
fn show_current_json_has_expected_shape() {
    let json = parse_clean_json(&run(&["show-current", "--json"]));

    let obj = json.as_object().expect("expected a JSON object");
    assert!(obj.contains_key("output"), "missing `output` key");
    assert!(obj.contains_key("input"), "missing `input` key");
}

#[test]
fn verbose_flag_does_not_pollute_json_stdout() {
    // Even with verbose logging enabled, `--json` must keep stdout clean.
    let json = parse_clean_json(&run(&["--verbose", "list-devices", "--json"]));
    assert!(json["devices"].is_array());
}
