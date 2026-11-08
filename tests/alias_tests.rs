use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(config: &Path, project: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_licens"))
        .current_dir(project)
        .env("APPDATA", config)
        .env("XDG_CONFIG_HOME", config)
        .env("HOME", config)
        .args(args)
        .output()
        .unwrap()
}

fn config_file(config: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    let config = config.join("Library/Application Support");
    config.join("licens/aliases.json")
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aliases_persist_across_projects_and_support_generation_flags() {
    let config = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    success(&run(config.path(), first.path(), &["assign", "m=MIT"]));
    for args in [
        vec![
            "M",
            "--author",
            "Alias Author",
            "--email",
            "a@example.com",
            "--year",
            "2030",
        ],
        vec![
            "generate",
            "m",
            "--author",
            "Alias Author",
            "--email",
            "a@example.com",
            "--year",
            "2030",
            "--force",
        ],
    ] {
        success(&run(config.path(), second.path(), &args));
        let content = fs::read_to_string(second.path().join("LICENSE")).unwrap();
        assert!(content.contains("MIT License"));
        assert!(content.contains("Alias Author <a@example.com>"));
        assert!(content.contains("2030"));
    }
    assert!(!first.path().join("LICENSE").exists());
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(config_file(config.path())).unwrap()).unwrap();
    assert_eq!(saved["m"], "mit");
}

#[test]
fn builtins_and_command_names_cannot_be_shadowed() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let reserved = licens::templates::LICENSES
        .iter()
        .flat_map(|template| [template.key, template.spdx_id, template.name])
        .chain(["list", "generate", "assign", "help", "version"]);
    for name in reserved {
        let assignment = format!("{}=mit", name.to_uppercase());
        let output = run(config.path(), project.path(), &["assign", &assignment]);
        assert!(!output.status.success(), "accepted reserved alias {name}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("reserved"));
    }
    assert!(!config_file(config.path()).exists());
}

#[test]
fn assignments_can_update_one_alias_without_losing_others() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for assignment in ["m=mit", "b=bsd-3-clause", "M=apache-2.0"] {
        success(&run(config.path(), project.path(), &["assign", assignment]));
    }
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(config_file(config.path())).unwrap()).unwrap();
    assert_eq!(saved["m"], "apache-2.0");
    assert_eq!(saved["b"], "bsd-3-clause");
}

#[test]
fn invalid_assignments_do_not_change_saved_aliases() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    success(&run(config.path(), project.path(), &["assign", "m=mit"]));
    let path = config_file(config.path());
    let original = fs::read(&path).unwrap();
    for assignment in [
        "missing-equals",
        "=mit",
        "1m=mit",
        "a/b=mit",
        "a b=mit",
        "x=",
        "x=nonexistent",
        "x=m",
        "m=mit=extra",
    ] {
        let output = run(config.path(), project.path(), &["assign", assignment]);
        assert!(!output.status.success(), "accepted {assignment}");
        assert_eq!(fs::read(&path).unwrap(), original);
    }
}

#[test]
fn corrupt_config_is_reported_without_breaking_builtin_generation() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let path = config_file(config.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"not JSON").unwrap();
    for args in [vec!["assign", "m=mit"], vec!["m"]] {
        let output = run(config.path(), project.path(), &args);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Cannot read aliases"));
        assert_eq!(fs::read(&path).unwrap(), b"not JSON");
    }
    success(&run(
        config.path(),
        project.path(),
        &[
            "mit",
            "--author",
            "Test",
            "--email",
            "test@example.com",
            "--year",
            "2026",
        ],
    ));
    assert!(project.path().join("LICENSE").exists());
}

#[test]
fn alias_generation_keeps_existing_file_protection() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    success(&run(config.path(), project.path(), &["assign", "m=mit"]));
    let path = project.path().join("LICENSE");
    fs::write(&path, b"keep this").unwrap();
    let output = run(
        config.path(),
        project.path(),
        &["m", "--author", "Test", "--email", "test@example.com"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--force"));
    assert_eq!(fs::read(path).unwrap(), b"keep this");
}
