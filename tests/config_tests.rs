use licens::config::AuthorInfo;
use licens::error::LicensError;
use std::collections::BTreeMap;

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

#[test]
fn saved_values_resolve_without_discovery_and_explicit_flags_win() {
    let preferences = BTreeMap::from([
        ("author".into(), " Saved Author ".into()),
        ("email".into(), " saved@example.com ".into()),
        ("year".into(), " 2030 ".into()),
    ]);
    let saved = AuthorInfo::resolve_with_preferences(None, None, None, &preferences).unwrap();
    assert_eq!(saved.name.as_deref(), Some("Saved Author"));
    assert_eq!(saved.email.as_deref(), Some("saved@example.com"));
    assert_eq!(saved.year, Some(2030));
    assert!(!saved.missing_author);
    let explicit = AuthorInfo::resolve_with_preferences(
        Some(" Flag Author ".into()),
        Some(" flag@example.com ".into()),
        Some(2042),
        &preferences,
    )
    .unwrap();
    assert_eq!(explicit.name.as_deref(), Some("Flag Author"));
    assert_eq!(explicit.email.as_deref(), Some("flag@example.com"));
    assert_eq!(explicit.year, Some(2042));
    assert!(!explicit.missing_author);
}

#[test]
fn ignored_fields_are_optional_and_explicit_values_override_them() {
    let mut preferences = BTreeMap::from([
        ("author".into(), "--ignore".into()),
        ("email".into(), "--ignore".into()),
        ("year".into(), "--ignore".into()),
    ]);
    let ignored = AuthorInfo::resolve_with_preferences(None, None, None, &preferences).unwrap();
    assert!(ignored.name.is_none());
    assert!(ignored.email.is_none());
    assert!(ignored.year.is_none());
    assert!(!ignored.missing_author);
    let explicit = AuthorInfo::resolve_with_preferences(
        Some("Override".into()),
        Some("override@example.com".into()),
        Some(-42),
        &preferences,
    )
    .unwrap();
    assert_eq!(explicit.name.as_deref(), Some("Override"));
    assert_eq!(explicit.email.as_deref(), Some("override@example.com"));
    assert_eq!(explicit.year, Some(-42));
    preferences.insert("email".into(), "email-only@example.com".into());
    let email_only = AuthorInfo::resolve_with_preferences(None, None, None, &preferences).unwrap();
    assert!(email_only.name.is_none());
    assert_eq!(email_only.email.as_deref(), Some("email-only@example.com"));
    assert!(!email_only.missing_author);
}

#[test]
fn invalid_saved_year_is_a_config_error_but_an_explicit_year_bypasses_it() {
    for year in ["not-a-year", "2147483648", "-2147483649"] {
        let preferences = BTreeMap::from([
            ("author".into(), "--ignore".into()),
            ("email".into(), "--ignore".into()),
            ("year".into(), year.into()),
        ]);
        assert!(matches!(
            AuthorInfo::resolve_with_preferences(None, None, None, &preferences),
            Err(LicensError::Config(_))
        ));
        let explicit =
            AuthorInfo::resolve_with_preferences(None, None, Some(2030), &preferences).unwrap();
        assert_eq!(explicit.year, Some(2030));
    }
}

#[test]
fn legacy_resolver_keeps_string_name_and_integer_year() {
    let author = AuthorInfo::resolve(
        Some("Legacy".into()),
        Some("legacy@example.com".into()),
        Some(1999),
    );
    let _: &String = &author.name;
    let _: &Option<String> = &author.email;
    let _: i32 = author.year;
    assert_eq!(author.name, "Legacy");
    assert_eq!(author.year, 1999);
    assert!(!author.missing_author);
}
