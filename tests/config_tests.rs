use licens::config::AuthorInfo;

#[test]
fn test_explicit_overrides_take_highest_precedence() {
    let author = AuthorInfo::resolve(
        Some("Explicit Author".to_string()),
        Some("explicit@example.com".to_string()),
        Some(2042),
    );

    assert_eq!(author.name, "Explicit Author");
    assert_eq!(author.email.as_deref(), Some("explicit@example.com"));
    assert_eq!(author.year, 2042);
}

#[test]
fn test_partial_override_keeps_provided_values() {
    let author = AuthorInfo::resolve(Some("Solo Author".to_string()), None, Some(1999));

    assert_eq!(author.name, "Solo Author");
    assert_eq!(author.year, 1999);
}

#[test]
fn test_default_resolution_does_not_panic() {
    let author = AuthorInfo::resolve(None, None, None);
    assert!(!author.name.is_empty());
    assert!(author.year >= 2024);
}
