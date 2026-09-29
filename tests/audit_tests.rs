mod helpers;

use helpers::TestWorkspace;
use licens::audit::model::{AuditReport, OutputFormat};
use licens::audit::{AuditReporter, CompatibilityAnalyzer, ManifestScanner};

#[test]
fn test_end_to_end_audit_multi_manifest_workspace() {
    let ws = TestWorkspace::new();

    // 1. Rust project manifest
    ws.write_cargo_toml(
        r#"
[package]
name = "full-stack-core"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
serde = "1.0"

[dev-dependencies]
criterion = "0.5"
"#,
    );

    // 2. Node project manifest in the same repo
    ws.write_package_json(
        r#"
{
  "name": "full-stack-web",
  "version": "1.0.0",
  "license": "Apache-2.0",
  "dependencies": {
    "react": "^18.2.0"
  },
  "devDependencies": {
    "eslint": "^8.0.0"
  }
}
"#,
    );

    // Scan prod-only dependencies
    let prod_scanned = ManifestScanner::scan_dir(ws.path(), false, false).unwrap();
    assert_eq!(prod_scanned.detected_license.as_deref(), Some("MIT"));
    assert_eq!(prod_scanned.dependencies.len(), 2); // serde + react

    // Scan with dev dependencies
    let all_scanned = ManifestScanner::scan_dir(ws.path(), true, false).unwrap();
    assert_eq!(all_scanned.dependencies.len(), 4); // serde + criterion + react + eslint

    // Manually assign licenses to simulate resolver results
    let mut resolved_deps = all_scanned.dependencies;
    for dep in &mut resolved_deps {
        match dep.name.as_str() {
            "serde" => dep.license = Some("MIT OR Apache-2.0".to_string()),
            "criterion" => dep.license = Some("Apache-2.0".to_string()),
            "react" => dep.license = Some("MIT".to_string()),
            "eslint" => dep.license = Some("MIT".to_string()),
            _ => dep.license = Some("Unknown".to_string()),
        }
    }

    let (conflicts, rec_perm, rec_restr) =
        CompatibilityAnalyzer::analyze(prod_scanned.detected_license.as_deref(), &resolved_deps);

    assert!(conflicts.is_empty());
    assert_eq!(rec_perm, "MIT");

    let report = AuditReport {
        project_license: prod_scanned.detected_license,
        total_dependencies: resolved_deps.len(),
        audited_dependencies: resolved_deps.len(),
        dependencies: resolved_deps,
        conflicts,
        recommended_permissive: rec_perm,
        recommended_restrictive: rec_restr,
    };

    // Test JSON report generation
    let json_output = AuditReporter::report(&report, OutputFormat::Json);
    assert!(json_output.contains("\"project_license\": \"MIT\""));
    assert!(json_output.contains("\"serde\""));
    assert!(json_output.contains("\"react\""));

    // Test Markdown report generation
    let md_output = AuditReporter::report(&report, OutputFormat::Markdown);
    assert!(md_output.contains("# Dependency License Audit Report"));
    assert!(md_output.contains("| serde |"));
    assert!(md_output.contains("| react |"));
    assert!(md_output.contains("✅ No License Conflicts Detected"));
}
