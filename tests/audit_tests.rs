use licens::audit::model::{ConflictSeverity, Dependency, DependencyKind};
use licens::audit::CompatibilityAnalyzer;

#[test]
fn test_compatibility_analyzer_conflict_detection() {
    let deps = vec![Dependency {
        name: "copyleft-lib".to_string(),
        version: Some("1.0.0".to_string()),
        kind: DependencyKind::Normal,
        source_manifest: "Cargo.toml".to_string(),
        license: Some("GPL-3.0".to_string()),
    }];

    let (conflicts, rec_permissive, rec_restrictive) =
        CompatibilityAnalyzer::analyze(Some("MIT"), &deps);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].severity, ConflictSeverity::Conflict);
    assert_eq!(rec_permissive, "GPL-3.0");
    assert_eq!(rec_restrictive, "GPL-3.0");
}

#[test]
fn test_compatibility_analyzer_permissive_recommendations() {
    let deps = vec![Dependency {
        name: "permissive-lib".to_string(),
        version: Some("2.0.0".to_string()),
        kind: DependencyKind::Normal,
        source_manifest: "Cargo.toml".to_string(),
        license: Some("MIT".to_string()),
    }];

    let (conflicts, rec_permissive, rec_restrictive) =
        CompatibilityAnalyzer::analyze(Some("MIT"), &deps);

    assert_eq!(conflicts.len(), 0);
    assert_eq!(rec_permissive, "MIT");
    assert_eq!(rec_restrictive, "GPL-3.0");
}
