mod helpers;

use helpers::TestWorkspace;
use licens::audit::model::DependencyKind;
use licens::audit::ManifestScanner;

#[test]
fn test_scan_cargo_toml_with_dev_and_build() {
    let ws = TestWorkspace::new();
    ws.write_cargo_toml(
        r#"
[package]
name = "sample-app"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
tokio = "1.38"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.10"

[build-dependencies]
cc = "1.0"
"#,
    );

    // Normal scan
    let normal_scan = ManifestScanner::scan_dir(ws.path(), false, false).unwrap();
    assert_eq!(normal_scan.detected_license.as_deref(), Some("MIT"));
    assert_eq!(normal_scan.dependencies.len(), 2);

    // Scan with dev and build
    let full_scan = ManifestScanner::scan_dir(ws.path(), true, true).unwrap();
    assert_eq!(full_scan.dependencies.len(), 4);

    let dev_dep = full_scan.dependencies.iter().find(|d| d.name == "tempfile").unwrap();
    assert_eq!(dev_dep.kind, DependencyKind::Dev);

    let build_dep = full_scan.dependencies.iter().find(|d| d.name == "cc").unwrap();
    assert_eq!(build_dep.kind, DependencyKind::Build);
}

#[test]
fn test_scan_cargo_toml_workspace_inheritance() {
    let ws = TestWorkspace::new();
    ws.write_cargo_toml(
        r#"
[workspace]
members = ["crates/*"]

[workspace.package]
license = "Apache-2.0"

[workspace.dependencies]
axum = "0.7"
"#,
    );

    let scanned = ManifestScanner::scan_dir(ws.path(), false, false).unwrap();
    assert_eq!(scanned.detected_license.as_deref(), Some("Apache-2.0"));
    assert_eq!(scanned.dependencies.len(), 1);
    assert_eq!(scanned.dependencies[0].name, "axum");
    assert_eq!(scanned.dependencies[0].source_manifest, "Cargo.toml (workspace)");
}

#[test]
fn test_scan_package_json_with_object_license() {
    let ws = TestWorkspace::new();
    ws.write_package_json(
        r#"
{
  "name": "node-project",
  "version": "1.0.0",
  "license": {
    "type": "BSD-3-Clause"
  },
  "dependencies": {
    "express": "^4.19.2"
  },
  "devDependencies": {
    "typescript": "^5.4.0"
  }
}
"#,
    );

    let scanned = ManifestScanner::scan_dir(ws.path(), true, false).unwrap();
    assert_eq!(scanned.detected_license.as_deref(), Some("BSD-3-Clause"));
    assert_eq!(scanned.dependencies.len(), 2);

    let ts = scanned.dependencies.iter().find(|d| d.name == "typescript").unwrap();
    assert_eq!(ts.kind, DependencyKind::Dev);
    assert_eq!(ts.version.as_deref(), Some("^5.4.0"));
}

#[test]
fn test_scan_pyproject_toml_pep621() {
    let ws = TestWorkspace::new();
    ws.write_pyproject_toml(
        r#"
[project]
name = "py-app"
version = "0.1.0"
license = { text = "MIT" }
dependencies = [
    "requests >= 2.28.0",
    "flask == 3.0.0"
]

[project.optional-dependencies]
dev = [
    "pytest >= 8.0"
]
"#,
    );

    let scanned = ManifestScanner::scan_dir(ws.path(), true, false).unwrap();
    assert_eq!(scanned.detected_license.as_deref(), Some("MIT"));
    assert_eq!(scanned.dependencies.len(), 3);

    let req = scanned.dependencies.iter().find(|d| d.name == "requests").unwrap();
    assert_eq!(req.version.as_deref(), Some(">= 2.28.0"));

    let pytest = scanned.dependencies.iter().find(|d| d.name == "pytest").unwrap();
    assert_eq!(pytest.kind, DependencyKind::Dev);
}

#[test]
fn test_scan_pyproject_toml_poetry() {
    let ws = TestWorkspace::new();
    ws.write_pyproject_toml(
        r#"
[tool.poetry]
name = "poetry-app"
version = "0.1.0"
license = "GPL-3.0"

[tool.poetry.dependencies]
python = "^3.11"
pydantic = "^2.6"
"#,
    );

    let scanned = ManifestScanner::scan_dir(ws.path(), false, false).unwrap();
    assert_eq!(scanned.detected_license.as_deref(), Some("GPL-3.0"));
    assert_eq!(scanned.dependencies.len(), 1);
    assert_eq!(scanned.dependencies[0].name, "pydantic");
}

#[test]
fn test_manifest_not_found_in_empty_dir() {
    let ws = TestWorkspace::new();
    let result = ManifestScanner::scan_dir(ws.path(), false, false);
    assert!(result.is_err());
}
