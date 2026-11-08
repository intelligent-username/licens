mod helpers;

use helpers::TestWorkspace;
use licens::commands::run_list;
use std::fs;

#[test]
fn test_cli_generate_creates_valid_license_file() {
    let ws = TestWorkspace::new();
    let target_path = ws.path().join("LICENSE");

    let output = command(&ws)
        .args([
            "mit",
            "--author",
            "Test Runner",
            "--email",
            "runner@example.com",
            "--year",
            "2026",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
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

    let output = command(&ws)
        .args([
            "apache-2.0",
            "--author",
            "Nested Corp",
            "--year",
            "2025",
            "--output",
        ])
        .arg(&nested_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(nested_path.exists());

    let content = fs::read_to_string(&nested_path).unwrap();
    assert!(content.contains("Apache License"));
    assert!(content.contains("2025"));
    assert!(content.contains("Nested Corp"));
}

#[test]
fn test_cli_generate_unsupported_license_returns_clean_error() {
    let ws = TestWorkspace::new();
    let output = command(&ws).arg("fake-license-xyz").output().unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Unsupported license: 'fake-license-xyz'")
    );
    assert!(!ws.path().join("LICENSE").exists());
}

#[test]
fn test_cli_list_runs_successfully() {
    assert!(run_list().is_ok());
}

fn command(ws: &TestWorkspace) -> std::process::Command {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_licens"));
    cmd.current_dir(ws.path())
        .env("APPDATA", ws.path().join("user-config"))
        .env("XDG_CONFIG_HOME", ws.path().join("user-config"))
        .env("HOME", ws.path().join("user-config"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", ws.path().join("no-gitconfig"))
        .env_remove("GIT_CONFIG_COUNT")
        .env_remove("GIT_CONFIG")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
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
    let local = ws.path().join(".git");
    fs::create_dir_all(local.join("objects")).unwrap();
    fs::create_dir_all(local.join("refs")).unwrap();
    fs::write(local.join("HEAD"), "ref: refs/heads/test\n").unwrap();
    fs::write(
        local.join("config"),
        "[core]\n repositoryformatversion = 0\n",
    )
    .unwrap();
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
    fs::write(
        local.join("config"),
        "[core]\n repositoryformatversion = 0\n[user]\n name = Local Name\n",
    )
    .unwrap();
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

#[cfg(any(unix, windows))]
#[test]
fn executable_force_rejects_symlink_destination() {
    let ws = TestWorkspace::new();
    let target = ws.path().join("original.txt");
    let link = ws.path().join("LICENSE");
    let original = b"Existing file must stay unchanged\n";
    fs::write(&target, original).unwrap();

    #[cfg(unix)]
    let result = std::os::unix::fs::symlink(&target, &link);
    #[cfg(windows)]
    let result = std::os::windows::fs::symlink_file(&target, &link);

    #[cfg(windows)]
    if result.as_ref().err().and_then(|e| e.raw_os_error()) == Some(1314) {
        eprintln!("Skipping symlink test: Windows requires Developer Mode or symlink privileges");
        return;
    }
    result.expect("failed to create test symlink");

    let output = command(&ws)
        .args(["mit", "--author", "Test", "--year", "2026", "--force"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a regular file"));
    assert_eq!(fs::read(&target).unwrap(), original);
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read_link(&link).unwrap(), target);
}
