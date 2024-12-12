use std::{fs::File, process::Command};

use serde_json::Value;

#[test]
fn csv_with_comma() {
    csv_cmd("test-comma", "test-data/comma/output.json");
}

#[test]
fn csv_with_semicolon_and_header_camel_case() {
    csv_cmd(
        "test-semicolon-camel",
        "test-data/semicolon-camel/output.json",
    );
}

#[test]
fn csv_with_semicolon_and_header_snake_case() {
    csv_cmd(
        "test-semicolon-snake",
        "test-data/semicolon-snake/output.json",
    );
}

#[test]
fn csv_with_transit_data() {
    csv_cmd(
        "test-transit",
        "test-data/transit/output.json",
    );
}

fn csv_cmd(arg: &str, output_file_path: &str) {
    // Read the expected output from a file
    let output_file = File::open(output_file_path).expect("file not found");
    let expected_output: serde_json::Value =
        serde_json::from_reader(output_file).expect("error while reading file");

    // Execute the command
    let output = Command::new("make")
        .arg(arg)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    println!("expected:\n{}", expected_output);
    println!("result:\n{}", stdout_str);
    
    let output_json: Value = serde_json::from_str(&stdout_str).expect("Failed to parse JSON");
    assert_eq!(output_json, expected_output);
}
