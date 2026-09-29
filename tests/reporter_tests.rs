mod helpers;

use helpers::make_dependency;
use licens::audit::model::{AuditReport, ConflictSeverity, DependencyKind, LicenseConflict, OutputFormat};
use licens::audit::AuditReporter;

#[test]
fn test_json_report_serialization() {
    let report = AuditReport {
        project_license: Some("MIT".to_string()),
        total_dependencies: 1,
        audited_dependencies: 1,
        dependencies: vec![make_dependency(
            "serde",
            Some("1.0.0"),
            Some("MIT OR Apache-2.0"),
            DependencyKind::Normal,
            "Cargo.toml",
        )],
        conflicts: vec![],
        recommended_permissive: "MIT".to_string(),
        recommended_restrictive: "GPL-3.0".to_string(),
    };

    let json_str = AuditReporter::report(&report, OutputFormat::Json);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Report should be valid JSON");

    assert_eq!(parsed["project_license"], "MIT");
    assert_eq!(parsed["total_dependencies"], 1);
    assert_eq!(parsed["dependencies"][0]["name"], "serde");
    assert_eq!(parsed["recommended_permissive"], "MIT");
}

#[test]
fn test_markdown_report_structure_with_conflicts() {
    let report = AuditReport {
        project_license: Some("MIT".to_string()),
        total_dependencies: 2,
        audited_dependencies: 2,
        dependencies: vec![
            make_dependency("tokio", Some("1.0.0"), Some("MIT"), DependencyKind::Normal, "Cargo.toml"),
            make_dependency("copyleft", Some("0.1.0"), Some("GPL-3.0"), DependencyKind::Normal, "Cargo.toml"),
        ],
        conflicts: vec![LicenseConflict {
            dependency_name: "copyleft".to_string(),
            dependency_license: "GPL-3.0".to_string(),
            project_license: "MIT".to_string(),
            severity: ConflictSeverity::Conflict,
            reason: "Viral copyleft conflict".to_string(),
        }],
        recommended_permissive: "GPL-3.0".to_string(),
        recommended_restrictive: "GPL-3.0".to_string(),
    };

    let md = AuditReporter::report(&report, OutputFormat::Markdown);

    assert!(md.contains("# Dependency License Audit Report"));
    assert!(md.contains("**Project License**: `MIT`"));
    assert!(md.contains("⚠️ Conflicts & Warnings"));
    assert!(md.contains("copyleft"));
    assert!(md.contains("Viral copyleft conflict"));
    assert!(md.contains("| Package | Version | Type | Manifest | License |"));
    assert!(md.contains("| tokio | 1.0.0 | prod | Cargo.toml | `MIT` |"));
    assert!(md.contains("**Most Permissive License**: `GPL-3.0`"));
}

#[test]
fn test_markdown_report_clean_state() {
    let report = AuditReport {
        project_license: Some("Apache-2.0".to_string()),
        total_dependencies: 1,
        audited_dependencies: 1,
        dependencies: vec![make_dependency("anyhow", Some("1.0.0"), Some("MIT"), DependencyKind::Normal, "Cargo.toml")],
        conflicts: vec![],
        recommended_permissive: "MIT".to_string(),
        recommended_restrictive: "GPL-3.0".to_string(),
    };

    let md = AuditReporter::report(&report, OutputFormat::Markdown);

    assert!(md.contains("✅ No License Conflicts Detected"));
    assert!(!md.contains("⚠️ Conflicts & Warnings"));
}
