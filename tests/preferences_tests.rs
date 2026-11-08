use chrono::Datelike;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const GIT_NAME: &str = "Git Discovery Private";
const GIT_EMAIL: &str = "git-private@example.com";
const PACKAGE_NAME: &str = "Package Discovery Private";
const PACKAGE_EMAIL: &str = "package-private@example.com";
const ENV_NAME: &str = "Environment Discovery Private";
const ENV_EMAIL: &str = "environment-private@example.com";

struct Fixture {
    config: TempDir,
    project: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let fixture = Self {
            config: tempfile::tempdir().unwrap(),
            project: tempfile::tempdir().unwrap(),
        };
        fs::write(
            fixture.git_config(),
            format!("[user]\n name = {GIT_NAME}\n email = {GIT_EMAIL}\n"),
        )
        .unwrap();
        fs::write(
            fixture.project.path().join("package.json"),
            format!(r#"{{"author":{{"name":"{PACKAGE_NAME}","email":"{PACKAGE_EMAIL}"}}}}"#),
        )
        .unwrap();
        fixture
    }

    fn path(&self) -> PathBuf {
        #[cfg(target_os = "macos")]
        let root = self.config.path().join("Library/Application Support");
        #[cfg(not(target_os = "macos"))]
        let root = self.config.path().to_path_buf();
        root.join("licens/config.json")
    }

    fn git_config(&self) -> PathBuf {
        self.config.path().join("identity.gitconfig")
    }

    fn command_in(&self, project: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_licens"));
        command
            .current_dir(project)
            .env("APPDATA", self.config.path())
            .env("XDG_CONFIG_HOME", self.config.path())
            .env("HOME", self.config.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.git_config())
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .env("GIT_AUTHOR_NAME", ENV_NAME)
            .env("GIT_AUTHOR_EMAIL", ENV_EMAIL)
            .env_remove("USER")
            .env_remove("USERNAME");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command_in(self.project.path())
            .args(args)
            .output()
            .unwrap()
    }

    fn configure(&self, assignments: &[&str]) {
        let output = self
            .command_in(self.project.path())
            .arg("config")
            .args(assignments)
            .output()
            .unwrap();
        success(&output);
    }

    fn write_config(&self, bytes: &[u8]) {
        let path = self.path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn content(&self, output: &str) -> String {
        fs::read_to_string(self.project.path().join(output)).unwrap()
    }
}

fn success(output: &Output) {
    assert!(output.status.success(), "{}", diagnostics(output));
}

fn rejection(output: &Output) {
    assert!(!output.status.success(), "{}", diagnostics(output));
    let text = diagnostics(output);
    for message in [
        "Successfully generated",
        "Configuration updated",
        "Assigned '",
    ] {
        assert!(!text.contains(message), "{text}");
    }
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn no_discovered_identity(text: &str) {
    for identity in [
        GIT_NAME,
        GIT_EMAIL,
        PACKAGE_NAME,
        PACKAGE_EMAIL,
        ENV_NAME,
        ENV_EMAIL,
    ] {
        assert!(!text.contains(identity), "leaked {identity}: {text}");
    }
}

#[test]
fn multiple_preferences_persist_across_projects_and_generation_is_read_only() {
    let fixture = Fixture::new();
    let output = fixture.run(&[
        "config",
        "author=Saved Person",
        "email=saved@example.com",
        "year=2030",
        "output=LICENSE.saved",
    ]);
    success(&output);
    assert!(!diagnostics(&output).contains("Saved Person"));
    assert!(!diagnostics(&output).contains("saved@example.com"));
    let bytes = fs::read(fixture.path()).unwrap();
    let saved: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(saved["preferences"]["author"], "Saved Person");
    assert_eq!(saved["preferences"]["email"], "saved@example.com");
    assert_eq!(saved["preferences"]["year"], "2030");
    assert_eq!(saved["preferences"]["output"], "LICENSE.saved");
    let other = tempfile::tempdir().unwrap();
    for project in [fixture.project.path(), other.path()] {
        success(&fixture.command_in(project).arg("mit").output().unwrap());
        let content = fs::read_to_string(project.join("LICENSE.saved")).unwrap();
        assert!(content.contains("Saved Person <saved@example.com>"));
        assert!(content.contains("2030"));
        no_discovered_identity(&content);
        assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
    }
}

#[test]
fn flags_override_saved_values_and_ignores_for_all_generation_entry_points() {
    let fixture = Fixture::new();
    success(&fixture.run(&["assign", "m=mit"]));
    let mut expected = None;
    for (batch, assignments) in [
        [
            "author=Saved",
            "email=saved@example.com",
            "year=2030",
            "output=saved/LICENSE",
        ],
        [
            "author=--ignore",
            "email=--ignore",
            "year=--ignore",
            "output=--ignore",
        ],
    ]
    .iter()
    .enumerate()
    {
        fixture.configure(assignments);
        let bytes = fs::read(fixture.path()).unwrap();
        for (index, entry) in [
            vec!["mit"],
            vec!["generate", "mit"],
            vec!["m"],
            vec!["generate", "m"],
        ]
        .iter()
        .enumerate()
        {
            let destination = format!("explicit-{batch}-{index}.txt");
            let output = fixture
                .command_in(fixture.project.path())
                .args(entry)
                .args([
                    "--author",
                    "Flag Person",
                    "--email",
                    "flag@example.com",
                    "--year",
                    "2042",
                    "--output",
                    &destination,
                ])
                .output()
                .unwrap();
            success(&output);
            let content = fixture.content(&destination);
            assert!(content.contains("Flag Person <flag@example.com>"));
            assert!(content.contains("2042"));
            no_discovered_identity(&content);
            if let Some(expected) = &expected {
                assert_eq!(&content, expected);
            } else {
                expected = Some(content);
            }
            assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
        }
    }
    assert!(!fixture.project.path().join("LICENSE").exists());
    assert!(!fixture.project.path().join("saved").exists());
}

#[test]
fn clearing_all_keys_restores_git_package_environment_year_and_output_fallbacks() {
    let fixture = Fixture::new();
    fixture.configure(&[
        "author=--ignore",
        "email=--ignore",
        "year=--ignore",
        "output=old/LICENSE",
    ]);
    fixture.configure(&["author=", "email=   ", "year=", "output= \t "]);
    let bytes = fs::read(fixture.path()).unwrap();
    let saved: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(saved["preferences"], serde_json::json!({}));
    for (name, email) in [
        (GIT_NAME, GIT_EMAIL),
        (PACKAGE_NAME, PACKAGE_EMAIL),
        (ENV_NAME, ENV_EMAIL),
    ] {
        success(&fixture.run(&["mit", "--force"]));
        let content = fixture.content("LICENSE");
        assert!(content.contains(&format!("{name} <{email}>")));
        assert!(content.contains(&chrono::Utc::now().year().to_string()));
        assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
        if name == GIT_NAME {
            fs::write(fixture.git_config(), "").unwrap();
        } else if name == PACKAGE_NAME {
            fs::remove_file(fixture.project.path().join("package.json")).unwrap();
        }
    }
    assert!(!fixture.project.path().join("old").exists());
    for variable in ["USER", "USERNAME"] {
        let output = fixture
            .command_in(fixture.project.path())
            .env_remove("GIT_AUTHOR_NAME")
            .env(variable, "User Fallback")
            .args(["mit", "--force"])
            .output()
            .unwrap();
        success(&output);
        assert!(fixture.content("LICENSE").contains("User Fallback"));
    }
}

#[test]
fn blank_saved_values_and_blank_flags_request_fallback_not_suppression() {
    let fixture = Fixture::new();
    fixture.write_config(br#"{"preferences":{"author":"  ","email":"","year":" ","output":" "}}"#);
    success(&fixture.run(&["mit"]));
    assert!(fixture
        .content("LICENSE")
        .contains(&format!("{GIT_NAME} <{GIT_EMAIL}>")));
    fixture.configure(&["author=--ignore", "email=--ignore", "output=not-used"]);
    success(&fixture.run(&[
        "mit", "--author", " ", "--email", "", "--output", "", "--force",
    ]));
    assert!(fixture
        .content("LICENSE")
        .contains(&format!("{GIT_NAME} <{GIT_EMAIL}>")));
    assert!(!fixture.project.path().join("not-used").exists());
}

#[test]
fn author_ignore_uses_bare_email_without_name_placeholder_or_warning() {
    let fixture = Fixture::new();
    fixture.configure(&["author=--ignore", "year=2030"]);
    let output = fixture.run(&["mit"]);
    success(&output);
    let text = format!("{}{}", fixture.content("LICENSE"), diagnostics(&output));
    assert!(text.contains(GIT_EMAIL));
    assert!(!text.contains(&format!("<{GIT_EMAIL}>")));
    for absent in [
        GIT_NAME,
        PACKAGE_NAME,
        ENV_NAME,
        "<author>",
        "[fullname]",
        "Author parameter is missing",
        "<>",
    ] {
        assert!(!text.contains(absent), "{text}");
    }
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!(" for {GIT_EMAIL} (2030).")));
}

#[test]
fn email_ignore_never_leaks_discovered_email_to_content_or_success_output() {
    let fixture = Fixture::new();
    fixture.configure(&["email=--ignore", "year=2030"]);
    let output = fixture.run(&["mit"]);
    success(&output);
    let content = fixture.content("LICENSE");
    assert!(content.contains(GIT_NAME));
    let text = format!("{content}{}", diagnostics(&output));
    for email in [GIT_EMAIL, PACKAGE_EMAIL, ENV_EMAIL, "<>"] {
        assert!(!text.contains(email), "{text}");
    }
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!(" for {GIT_NAME} (2030).")));
}

#[test]
fn ignored_identity_and_year_have_no_discovery_warning_tokens_or_success_suffixes() {
    let fixture = Fixture::new();
    fixture.configure(&["author=--ignore", "email=--ignore", "year=--ignore"]);
    let output = fixture.run(&["mit"]);
    success(&output);
    let content = fixture.content("LICENSE");
    let text = format!("{content}{}", diagnostics(&output));
    no_discovered_identity(&text);
    for absent in [
        "<author>",
        "[author]",
        "[fullname]",
        "[year]",
        "[Year]",
        "<YEAR>",
        "<year>",
        "[yyyy]",
        "Author parameter is missing",
        "<>",
        "()",
    ] {
        assert!(!text.contains(absent), "{text}");
    }
    assert!(!content.contains(&chrono::Utc::now().year().to_string()));
    for line in content.lines().filter(|line| line.contains("Copyright")) {
        assert!(!line.split_whitespace().any(|word| word == "0"), "{line}");
    }
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Successfully generated MIT file at 'LICENSE'."
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn year_ignore_is_independent_of_identity() {
    let fixture = Fixture::new();
    fixture.configure(&["year=--ignore"]);
    let output = fixture.run(&["mit"]);
    success(&output);
    let content = fixture.content("LICENSE");
    assert!(content.contains(&format!("{GIT_NAME} <{GIT_EMAIL}>")));
    assert!(!content.contains(&chrono::Utc::now().year().to_string()));
    for token in ["[year]", "[Year]", "<YEAR>", "<year>", "[yyyy]"] {
        assert!(!content.contains(token));
    }
    for line in content.lines().filter(|line| line.contains("Copyright")) {
        assert!(!line.split_whitespace().any(|word| word == "0"));
    }
    assert!(String::from_utf8_lossy(&output.stdout)
        .ends_with(&format!(" for {GIT_NAME} <{GIT_EMAIL}>.\n")));
}

#[test]
fn saved_output_is_project_relative_and_keeps_existing_file_protection() {
    let fixture = Fixture::new();
    fixture.configure(&["output=deep/nested/LICENSE.md"]);
    let other = tempfile::tempdir().unwrap();
    let destination = other.path().join("deep/nested/LICENSE.md");
    success(
        &fixture
            .command_in(other.path())
            .arg("mit")
            .output()
            .unwrap(),
    );
    assert!(!fixture.project.path().join("deep").exists());
    assert!(!fixture.config.path().join("deep").exists());
    let original = fs::read(&destination).unwrap();
    let refused = fixture
        .command_in(other.path())
        .args(["mit", "--author", "Changed"])
        .output()
        .unwrap();
    rejection(&refused);
    assert!(diagnostics(&refused).contains("--force"));
    assert_eq!(fs::read(&destination).unwrap(), original);
    success(
        &fixture
            .command_in(other.path())
            .args(["mit", "--author", "Changed", "--force"])
            .output()
            .unwrap(),
    );
    assert!(fs::read_to_string(&destination)
        .unwrap()
        .contains("Changed"));
    success(&fixture.run(&["mit", "--output", "explicit.txt"]));
    assert!(fixture.project.path().join("explicit.txt").exists());
    assert!(!fixture.project.path().join("deep").exists());
}

#[test]
fn absent_and_ignored_output_both_use_license_without_persisting_discovery() {
    let fixture = Fixture::new();
    success(&fixture.run(&["mit"]));
    assert!(fixture.project.path().join("LICENSE").exists());
    assert!(!fixture.path().exists());
    fixture.configure(&["output=--ignore"]);
    let original = fs::read(fixture.path()).unwrap();
    success(&fixture.run(&["mit", "--force"]));
    assert_eq!(fs::read(fixture.path()).unwrap(), original);
    assert!(!fixture.project.path().join("--ignore").exists());
}

#[test]
fn invalid_batches_are_atomic_with_existing_and_absent_configuration() {
    for existing in [false, true] {
        let fixture = Fixture::new();
        if existing {
            fixture.configure(&["author=Original", "email=--ignore"]);
        }
        let original = fs::read(fixture.path()).ok();
        for invalid in [
            "unknown=value",
            "missing-equals",
            "Author=Wrong",
            "=value",
            "year=invalid",
            "year=2147483648",
            "year=-2147483649",
        ] {
            let output = fixture.run(&["config", "author=Changed", invalid, "year=2030"]);
            rejection(&output);
            assert_eq!(fs::read(fixture.path()).ok(), original, "{invalid}");
        }
        rejection(&fixture.run(&["config"]));
        assert_eq!(fs::read(fixture.path()).ok(), original);
    }
}

#[test]
fn assignments_keep_equals_trim_values_and_apply_repeated_keys_in_order() {
    let fixture = Fixture::new();
    fixture.configure(&[
        " author = First ",
        "author=Last=Person",
        "email=a=b@example.com",
        "year=-2147483648",
    ]);
    let saved: Value = serde_json::from_slice(&fs::read(fixture.path()).unwrap()).unwrap();
    assert_eq!(saved["preferences"]["author"], "Last=Person");
    assert_eq!(saved["preferences"]["email"], "a=b@example.com");
    assert_eq!(saved["preferences"]["year"], "-2147483648");
    fixture.configure(&["year=2147483647", "output=kept.txt"]);
    fixture.configure(&["author=Changed", "author=", "year=--ignore"]);
    let saved: Value = serde_json::from_slice(&fs::read(fixture.path()).unwrap()).unwrap();
    assert!(saved["preferences"].get("author").is_none());
    assert_eq!(saved["preferences"]["email"], "a=b@example.com");
    assert_eq!(saved["preferences"]["output"], "kept.txt");
    assert_eq!(saved["preferences"]["year"], "--ignore");
}

#[test]
fn malformed_new_config_fails_closed_for_generation_and_mutations_preserve_bytes() {
    let fixture = Fixture::new();
    let existing = fixture.project.path().join("existing.txt");
    fs::write(&existing, b"keep existing destination").unwrap();
    for invalid in [
        "not JSON",
        "[]",
        r#"{"extra":{}}"#,
        r#"{"aliases":null}"#,
        r#"{"aliases":{"m":1}}"#,
        r#"{"preferences":[]}"#,
        r#"{"preferences":{"author":false}}"#,
        r#"{"preferences":{"unknown":"value"}}"#,
        r#"{"preferences":{"year":"2147483648"}}"#,
    ] {
        fixture.write_config(invalid.as_bytes());
        for entry in [
            vec!["mit"],
            vec!["generate", "mit"],
            vec!["m"],
            vec!["generate", "m"],
        ] {
            for destination in ["new/nested/LICENSE", "existing.txt"] {
                let output = fixture
                    .command_in(fixture.project.path())
                    .args(&entry)
                    .args(["--output", destination, "--force"])
                    .output()
                    .unwrap();
                rejection(&output);
                let text = diagnostics(&output);
                assert!(text.contains("Cannot use configuration"), "{text}");
                assert!(text.contains("config.json"), "{text}");
                no_discovered_identity(&text);
                assert!(!fixture.project.path().join("new").exists());
                assert_eq!(fs::read(&existing).unwrap(), b"keep existing destination");
                assert_eq!(fs::read(fixture.path()).unwrap(), invalid.as_bytes());
            }
        }
        for args in [vec!["config", "author=Changed"], vec!["assign", "m=mit"]] {
            rejection(&fixture.run(&args));
            assert_eq!(fs::read(fixture.path()).unwrap(), invalid.as_bytes());
        }
    }
}

#[test]
fn unreadable_config_and_invalid_roots_fail_before_output_creation() {
    let fixture = Fixture::new();
    let path = fixture.path();
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("keep"), b"directory destination stays").unwrap();
    for args in [
        vec!["mit"],
        vec!["m"],
        vec!["config", "author=Changed"],
        vec!["assign", "m=mit"],
    ] {
        let output = fixture.run(&args);
        rejection(&output);
        assert!(diagnostics(&output).contains("Cannot use configuration"));
        no_discovered_identity(&diagnostics(&output));
        assert_eq!(
            fs::read(path.join("keep")).unwrap(),
            b"directory destination stays"
        );
        assert!(!fixture.project.path().join("LICENSE").exists());
    }
    let blocked = fixture.config.path().join("not-a-directory");
    fs::write(&blocked, b"keep root file").unwrap();
    for root in [Path::new("relative-config-root"), blocked.as_path()] {
        for entry in ["mit", "m"] {
            let output = fixture
                .command_in(fixture.project.path())
                .env("APPDATA", root)
                .env("XDG_CONFIG_HOME", root)
                .env("HOME", root)
                .args([entry, "--output", "uncreated/LICENSE", "--force"])
                .output()
                .unwrap();
            rejection(&output);
            no_discovered_identity(&diagnostics(&output));
            assert!(diagnostics(&output).contains("configuration"));
            assert!(!fixture.project.path().join("uncreated").exists());
        }
    }
    assert_eq!(fs::read(blocked).unwrap(), b"keep root file");
    assert!(!fixture.project.path().join("relative-config-root").exists());
}

#[test]
fn repeated_saves_preserve_aliases_and_unrelated_preferences() {
    let fixture = Fixture::new();
    fixture.configure(&["author=Original", "email=--ignore", "output=LICENSE.md"]);
    success(&fixture.run(&["assign", "m=mit"]));
    fixture.configure(&["year=2030"]);
    success(&fixture.run(&["assign", "b=bsd-3-clause"]));
    fixture.configure(&["author=Final"]);
    let saved: Value = serde_json::from_slice(&fs::read(fixture.path()).unwrap()).unwrap();
    assert_eq!(
        saved["aliases"],
        serde_json::json!({"m":"mit","b":"bsd-3-clause"})
    );
    assert_eq!(
        saved["preferences"],
        serde_json::json!({
            "author":"Final", "email":"--ignore", "output":"LICENSE.md", "year":"2030",
        })
    );
}
