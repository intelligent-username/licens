mod helpers;

use helpers::make_dependency;
use licens::audit::model::{ConflictSeverity, DependencyKind};
use licens::audit::CompatibilityAnalyzer;

#[test]
fn test_permissive_project_with_gpl3_triggers_conflict() {
    let deps = vec![make_dependency(
        "gpl-crate",
        Some("1.0.0"),
        Some("GPL-3.0"),
        DependencyKind::Normal,
        "Cargo.toml",
    )];

    let (conflicts, rec_perm, rec_restr) = CompatibilityAnalyzer::analyze(Some("MIT"), &deps);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].dependency_name, "gpl-crate");
    assert_eq!(conflicts[0].severity, ConflictSeverity::Conflict);
    assert!(conflicts[0].reason.contains("strong copyleft"));
    assert_eq!(rec_perm, "GPL-3.0");
    assert_eq!(rec_restr, "GPL-3.0");
}

#[test]
fn test_permissive_project_with_gpl2_recommends_gpl2() {
    let deps = vec![make_dependency(
        "legacy-gpl",
        Some("0.5.0"),
        Some("GPL-2.0"),
        DependencyKind::Normal,
        "Cargo.toml",
    )];

    let (conflicts, rec_perm, rec_restr) = CompatibilityAnalyzer::analyze(Some("Apache-2.0"), &deps);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(rec_perm, "GPL-2.0");
    assert_eq!(rec_restr, "GPL-2.0");
}

#[test]
fn test_gpl2_project_conflicts_with_apache2_patents() {
    let deps = vec![make_dependency(
        "apache-dep",
        Some("2.4.0"),
        Some("Apache-2.0"),
        DependencyKind::Normal,
        "Cargo.toml",
    )];

    let (conflicts, _, _) = CompatibilityAnalyzer::analyze(Some("gpl-2.0"), &deps);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].severity, ConflictSeverity::Conflict);
    assert!(conflicts[0].reason.contains("patent"));
}

#[test]
fn test_gpl3_project_allows_apache2() {
    let deps = vec![make_dependency(
        "apache-dep",
        Some("2.4.0"),
        Some("Apache-2.0"),
        DependencyKind::Normal,
        "Cargo.toml",
    )];

    let (conflicts, _, _) = CompatibilityAnalyzer::analyze(Some("gpl-3.0"), &deps);

    assert!(conflicts.is_empty(), "GPL-3.0 is compatible with Apache-2.0");
}

#[test]
fn test_all_permissive_dependencies_no_conflicts() {
    let deps = vec![
        make_dependency("dep-a", Some("1.0.0"), Some("MIT"), DependencyKind::Normal, "Cargo.toml"),
        make_dependency("dep-b", Some("2.0.0"), Some("Apache-2.0"), DependencyKind::Normal, "package.json"),
        make_dependency("dep-c", Some("0.1.0"), Some("BSD-3-Clause"), DependencyKind::Normal, "pyproject.toml"),
        make_dependency("dep-d", Some("1.2.0"), Some("ISC"), DependencyKind::Normal, "Cargo.toml"),
    ];

    let (conflicts, rec_perm, rec_restr) = CompatibilityAnalyzer::analyze(Some("MIT"), &deps);

    assert!(conflicts.is_empty());
    assert_eq!(rec_perm, "MIT");
    assert_eq!(rec_restr, "GPL-3.0");
}

#[test]
fn test_weak_copyleft_advises_dynamic_linking() {
    let deps = vec![make_dependency(
        "mpl-dep",
        Some("1.0.0"),
        Some("MPL-2.0"),
        DependencyKind::Normal,
        "Cargo.toml",
    )];

    let (_, rec_perm, _) = CompatibilityAnalyzer::analyze(Some("MIT"), &deps);

    assert!(rec_perm.contains("dynamic linking"));
}

#[test]
fn test_empty_dependencies_produces_standard_recommendations() {
    let (conflicts, rec_perm, rec_restr) = CompatibilityAnalyzer::analyze(Some("MIT"), &[]);

    assert!(conflicts.is_empty());
    assert_eq!(rec_perm, "MIT");
    assert_eq!(rec_restr, "GPL-3.0");
}
