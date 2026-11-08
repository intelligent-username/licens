use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(config: &Path, project: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_licens"))
        .current_dir(project)
        .env("APPDATA", config)
        .env("XDG_CONFIG_HOME", config)
        .env("HOME", config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", config.join("no-gitconfig"))
        .env_remove("GIT_CONFIG_COUNT")
        .env_remove("GIT_CONFIG")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_AUTHOR_NAME")
        .env_remove("GIT_AUTHOR_EMAIL")
        .env_remove("USER")
        .env_remove("USERNAME")
        .args(args)
        .output()
        .unwrap()
}

fn config_file(config: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    let config = config.join("Library/Application Support");
    config.join("licens/config.json")
}

fn legacy_file(config: &Path) -> PathBuf {
    config_file(config).with_file_name("aliases.json")
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
    assert_eq!(saved["aliases"]["m"], "mit");
}

#[test]
fn builtins_and_command_names_cannot_be_shadowed() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let reserved = licens::templates::LICENSES
        .iter()
        .flat_map(|template| [template.key, template.spdx_id, template.name])
        .chain(["list", "generate", "assign", "config", "help", "version"]);
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
    assert_eq!(saved["aliases"]["m"], "apache-2.0");
    assert_eq!(saved["aliases"]["b"], "bsd-3-clause");
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
fn corrupt_legacy_aliases_fail_aliases_but_allow_builtin_generation() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let path = legacy_file(config.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"not JSON").unwrap();
    for args in [
        vec!["assign", "m=mit"],
        vec!["config", "email=--ignore"],
        vec!["m"],
    ] {
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
    assert_eq!(fs::read(path).unwrap(), b"not JSON");
    assert!(!config_file(config.path()).exists());
}

#[test]
fn legacy_import_is_read_only_then_migrates_without_losing_preferences() {
    let config = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let legacy = legacy_file(config.path());
    let bytes = b"{ \"m\": \"mit\", \"config\": \"mit\" }\n";
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, bytes).unwrap();

    for (project, args) in [
        (
            first.path(),
            vec!["m", "--author", "Legacy", "--year", "2030"],
        ),
        (
            second.path(),
            vec!["generate", "m", "--author", "Legacy", "--year", "2030"],
        ),
    ] {
        success(&run(config.path(), project, &args));
        assert!(fs::read_to_string(project.join("LICENSE"))
            .unwrap()
            .contains("Legacy"));
        assert!(!config_file(config.path()).exists());
        assert_eq!(fs::read(&legacy).unwrap(), bytes);
    }

    success(&run(
        config.path(),
        first.path(),
        &["config", "author=Saved", "email=--ignore"],
    ));
    success(&run(
        config.path(),
        first.path(),
        &["assign", "b=bsd-3-clause"],
    ));
    success(&run(config.path(), first.path(), &["config", "year=2031"]));
    success(&run(
        config.path(),
        first.path(),
        &["assign", "m=apache-2.0"],
    ));
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(config_file(config.path())).unwrap()).unwrap();
    assert_eq!(saved["aliases"]["m"], "apache-2.0");
    assert_eq!(saved["aliases"]["b"], "bsd-3-clause");
    assert_eq!(saved["aliases"]["config"], "mit");
    assert_eq!(saved["preferences"]["author"], "Saved");
    assert_eq!(saved["preferences"]["email"], "--ignore");
    assert_eq!(saved["preferences"]["year"], "2031");
    assert_eq!(fs::read(legacy).unwrap(), bytes);
    success(&run(config.path(), second.path(), &["m", "--force"]));
    let content = fs::read_to_string(second.path().join("LICENSE")).unwrap();
    assert!(content.contains("Apache License"));
    assert!(content.contains("Saved"));
    assert!(content.contains("2031"));
}

#[test]
fn alias_mutation_imports_legacy_and_new_config_does_not_resurrect_stale_aliases() {
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let legacy = legacy_file(config.path());
    let bytes = b"{\"old\":\"mit\"}\n";
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, bytes).unwrap();
    success(&run(
        config.path(),
        project.path(),
        &["assign", "new=bsd-3-clause"],
    ));
    let path = config_file(config.path());
    let mut saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["aliases"]["old"], "mit");
    assert_eq!(saved["aliases"]["new"], "bsd-3-clause");
    saved["aliases"].as_object_mut().unwrap().remove("old");
    let authoritative = serde_json::to_vec(&saved).unwrap();
    fs::write(&path, &authoritative).unwrap();
    let output = run(config.path(), project.path(), &["old"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unsupported license"));
    assert!(!project.path().join("LICENSE").exists());
    assert_eq!(fs::read(path).unwrap(), authoritative);
    assert_eq!(fs::read(legacy).unwrap(), bytes);
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

#[test]
fn list_shows_saved_aliases_beside_originals_without_color_when_captured() {
    let config = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    for assignment in ["m=mit", "mine=mit", "a=apache-2.0"] {
        success(&run(config.path(), first.path(), &["assign", assignment]));
    }
    let path = config_file(config.path());
    let original_config = fs::read(&path).unwrap();
    let output = run(config.path(), second.path(), &["list"]);
    success(&output);
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("ORIGINAL") && text.contains("ALIASES"));
    let mit_row = text.lines().find(|row| row.starts_with("mit ")).unwrap();
    assert!(mit_row.contains("m, mine"));
    assert!(text
        .lines()
        .any(|row| row.starts_with("apache-2.0") && row.contains(" a ")));
    assert!(!text.contains('\x1b'));
    assert_eq!(fs::read(path).unwrap(), original_config);
}
