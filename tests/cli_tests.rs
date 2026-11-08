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
        force: false,
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
        force: false,
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
        force: false,
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

fn command(ws: &TestWorkspace) -> std::process::Command {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_licens"));
    cmd.current_dir(ws.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", ws.path().join("no-gitconfig"))
        .env_remove("GIT_AUTHOR_NAME")
        .env_remove("GIT_AUTHOR_EMAIL")
        .env_remove("USER")
        .env_remove("USERNAME");
    cmd
}

#[test]
fn executable_generation_overwrite_and_flags() {
    let ws = TestWorkspace::new();
    let output = command(&ws)
        .args(["mit", "--author", "Ada", "--year", "2026"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let path = ws.path().join("LICENSE");
    let original = fs::read(&path).unwrap();
    let output = command(&ws)
        .args(["generate", "mit", "--author", "New", "--year", "2027"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--force"));
    assert_eq!(fs::read(&path).unwrap(), original);
    let output = command(&ws)
        .args([
            "generate", "mit", "--author", "New", "--year", "2027", "--force",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let changed = fs::read_to_string(&path).unwrap();
    assert!(changed.contains("New") && changed.contains("2027"));
    assert!(!command(&ws)
        .args(["mit", "extra"])
        .status()
        .unwrap()
        .success());
}

#[test]
fn executable_missing_author_warns_and_uses_current_year() {
    let ws = TestWorkspace::new();
    let output = command(&ws).arg("mit").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("Author parameter is missing"));
    let content = fs::read_to_string(ws.path().join("LICENSE")).unwrap();
    assert!(content.contains("<author>"));
    assert!(content.contains(&chrono::Datelike::year(&chrono::Utc::now()).to_string()));
}

#[test]
fn executable_package_author_and_malformed_author() {
    let ws = TestWorkspace::new();
    fs::write(
        ws.path().join("package.json"),
        r#"{"author":{"name":"Package Author","email":"a@example.com"}}"#,
    )
    .unwrap();
    let output = command(&ws)
        .args(["mit", "--year", "2026"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(fs::read_to_string(ws.path().join("LICENSE"))
        .unwrap()
        .contains("Package Author <a@example.com>"));
    fs::write(ws.path().join("package.json"), r#"{"author":">broken<"}"#).unwrap();
    let output = command(&ws).args(["mit", "--force"]).output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Author parameter is missing"));
}

#[test]
fn executable_git_local_and_global_identity() {
    let ws = TestWorkspace::new();
    let git = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(ws.path())
        .status()
        .unwrap();
    assert!(git.success());
    let global = ws.path().join("global.gitconfig");
    fs::write(
        &global,
        "[user]\n name = Global Name\n email = global@example.com\n",
    )
    .unwrap();
    let output = command(&ws)
        .env("GIT_CONFIG_GLOBAL", &global)
        .args(["mit", "--year", "2026"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(fs::read_to_string(ws.path().join("LICENSE"))
        .unwrap()
        .contains("Global Name <global@example.com>"));
    let status = std::process::Command::new("git")
        .args(["config", "--local", "user.name", "Local Name"])
        .current_dir(ws.path())
        .status()
        .unwrap();
    assert!(status.success());
    let output = command(&ws)
        .env("GIT_CONFIG_GLOBAL", &global)
        .args(["mit", "--force"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(fs::read_to_string(ws.path().join("LICENSE"))
        .unwrap()
        .contains("Local Name <global@example.com>"));
}

#[test]
fn executable_help_list_errors_and_nested_output() {
    let ws = TestWorkspace::new();
    for arg in ["--help", "--version", "list"] {
        let output = command(&ws).arg(arg).output().unwrap();
        assert!(output.status.success());
        if arg == "list" {
            let text = String::from_utf8(output.stdout).unwrap();
            for template in licens::templates::LICENSES {
                assert!(text.contains(template.key));
            }
        }
    }
    let output = command(&ws)
        .args([
            "generate",
            "mit",
            "--author",
            "Test",
            "--year",
            "2026",
            "--output",
            "nested/LICENSE",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(ws.path().join("nested/LICENSE").exists());
    for args in [
        &["missing-license"][..],
        &["mit", "--bogus"][..],
        &["mit", "--output", "nested"][..],
    ] {
        assert!(!command(&ws).args(args).output().unwrap().status.success());
    }
}
