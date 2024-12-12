use std::process::Command;
use std::fs;

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

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn csv_cmd(arg: &str, output_file_path: &str) {
    let expected = fs::read_to_string(output_file_path).unwrap();

    // Execute the command
    let output = Command::new("make")
        .arg(arg)
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);

    println!("expected:\n{:?}", output);
    println!("result:\n{:?}", result.as_ref());
    
    assert_eq!(
        remove_whitespace(expected.as_str()), 
        remove_whitespace(result.as_ref()));
}
