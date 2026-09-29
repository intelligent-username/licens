mod helpers;

use helpers::TestWorkspace;
use licens::commands::{run_generate, run_list, GenerateOptions};
use licens::error::LicensError;
use std::fs;

#[test]
fn test_cli_generate_creates_valid_license_file() {
    let ws = TestWorkspace::new();
    let target_path = ws.path().join("LICENSE");

    let opts = GenerateOptions {
        license_key: "mit".to_string(),
        output_path: Some(target_path.to_string_lossy().to_string()),
        author_name: Some("Test Runner".to_string()),
        author_email: Some("runner@example.com".to_string()),
        year: Some(2026),
    };

    let result = run_generate(opts);
    assert!(result.is_ok());
    assert!(target_path.exists());

    let content = fs::read_to_string(&target_path).expect("Failed to read generated license");
    assert!(content.contains("MIT License"));
    assert!(content.contains("2026"));
    assert!(content.contains("Test Runner <runner@example.com>"));
}

#[test]
fn test_cli_generate_creates_nested_directories_automatically() {
    let ws = TestWorkspace::new();
    let nested_path = ws.path().join("deep").join("nested").join("LICENSE.md");

    let opts = GenerateOptions {
        license_key: "apache-2.0".to_string(),
        output_path: Some(nested_path.to_string_lossy().to_string()),
        author_name: Some("Nested Corp".to_string()),
        author_email: None,
        year: Some(2025),
    };

    let result = run_generate(opts);
    assert!(result.is_ok());
    assert!(nested_path.exists());

    let content = fs::read_to_string(&nested_path).unwrap();
    assert!(content.contains("Apache License"));
    assert!(content.contains("2025"));
    assert!(content.contains("Nested Corp"));
}

#[test]
fn test_cli_generate_unsupported_license_returns_clean_error() {
    let opts = GenerateOptions {
        license_key: "fake-license-xyz".to_string(),
        output_path: None,
        author_name: None,
        author_email: None,
        year: None,
    };

    let result = run_generate(opts);
    assert!(result.is_err());
    match result.unwrap_err() {
        LicensError::UnsupportedLicense(key) => {
            assert_eq!(key, "fake-license-xyz");
        }
        other => panic!("Expected UnsupportedLicense error, got {:?}", other),
    }
}

#[test]
fn test_cli_list_runs_successfully() {
    assert!(run_list().is_ok());
}
