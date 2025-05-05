use std::fs;
use std::process::Command;

#[test]
fn casing_default() {
    make_cmd("test-casing-default", "test-data/output-snake.json");
}

#[test]
fn max_depth() {
    make_cmd("test-casing-depth", "test-data/output-depth.json");
}

#[test]
fn max_array() {
    make_cmd("test-casing-array", "test-data/output-array.json");
}

#[test]
fn casing_camel() {
    make_cmd("test-casing-camel", "test-data/output-camel.json");
}

#[test]
fn casing_pascal() {
    make_cmd("test-casing-pascal", "test-data/output-pascal.json");
}

#[test]
fn casing_kebab() {
    make_cmd("test-casing-kebab", "test-data/output-kebab.json");
}

#[test]
fn casing_const() {
    make_cmd("test-casing-const", "test-data/output-const.json");
}

#[test]
fn casing_cobol() {
    make_cmd("test-casing-cobol", "test-data/output-cobol.json");
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn make_cmd(arg: &str, output_file_path: &str) {
    let expected = fs::read_to_string(output_file_path).unwrap();

    // Execute the command
    let output = Command::new("make")
        .arg(arg)
        .output()
        .expect("cannot execute command");
    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);

    println!("expected:\n{:?}", output);
    println!("result:\n{:?}", result.as_ref());

    assert_eq!(
        remove_whitespace(expected.as_str()),
        remove_whitespace(result.as_ref())
    );
}
