use std::process::Command;

#[test]
fn test_end_to_end_sample_obfuscated() {
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", "lunadec-cli", "--", "examples/sample_obfuscated.lua"])
        .output()
        .expect("Failed to execute lunadec-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check for our normalized expected output from stage 7 dynamically verified without fixed exact layouts:
    assert!(stdout.contains("greeting"));
    assert!(stdout.contains("decode"));
    assert!(stdout.contains("_msg"));
}
