use licens::templates::{find_license, LicenseCategory, LICENSES};

#[test]
fn test_all_catalog_licenses_resolve() {
    let expected_keys = [
        "mit",
        "apache-2.0",
        "gpl-2.0",
        "gpl-3.0",
        "agpl-3.0",
        "lgpl-2.1",
        "mpl-2.0",
        "bsd-1-clause",
        "bsd-2-clause",
        "bsd-3-clause",
        "bsl-1.0",
        "epl-2.0",
        "isc",
        "cc0-1.0",
        "unlicense",
    ];

    for key in expected_keys {
        let lic = find_license(key);
        assert!(lic.is_some(), "License key '{key}' must resolve");
        let lic = lic.unwrap();
        assert!(!lic.name.is_empty());
        assert!(!lic.spdx_id.is_empty());
    }
}

#[test]
fn test_case_insensitive_lookup() {
    assert!(find_license("MIT").is_some());
    assert!(find_license("Mit").is_some());
    assert!(find_license("APACHE-2.0").is_some());
    assert!(find_license("Gpl-3.0").is_some());
    assert!(find_license("bsd-2-CLAUSE").is_some());
}

#[test]
fn test_spdx_id_lookup() {
    assert_eq!(
        find_license("BSD-3-Clause").map(|l| l.key),
        Some("bsd-3-clause")
    );
    assert_eq!(
        find_license("Apache-2.0").map(|l| l.key),
        Some("apache-2.0")
    );
    assert_eq!(find_license("CC0-1.0").map(|l| l.key), Some("cc0-1.0"));
}

#[test]
fn test_rendering_interpolates_all_placeholders() {
    let year = 2026;
    let author = "Ada Lovelace <ada@example.com>";

    for lic in LICENSES {
        assert!(!lic.text.trim().is_empty(), "{} is empty", lic.key);

        let rendered = lic.render(year, author);

        assert!(
            !rendered.contains("[year]"),
            "License '{}' left '[year]' unrendered",
            lic.key
        );
        assert!(
            !rendered.contains("[author]"),
            "License '{}' left '[author]' unrendered",
            lic.key
        );
        assert!(
            !rendered.contains("[fullname]"),
            "License '{}' left '[fullname]' unrendered",
            lic.key
        );
    }
}

#[test]
fn catalog_keys_and_spdx_identifiers_are_unique() {
    let mut keys = std::collections::HashSet::new();
    let mut identifiers = std::collections::HashSet::new();
    for lic in LICENSES {
        assert!(keys.insert(lic.key.to_ascii_lowercase()));
        assert!(identifiers.insert(lic.spdx_id.to_ascii_lowercase()));
    }
    assert_eq!(LICENSES.len(), 15);
}

#[test]
fn test_license_category_classifications() {
    let mit = find_license("mit").unwrap();
    assert_eq!(mit.category, LicenseCategory::Permissive);
    assert_eq!(mit.category.as_str(), "Permissive");

    let gpl3 = find_license("gpl-3.0").unwrap();
    assert_eq!(gpl3.category, LicenseCategory::CopyleftStrong);
    assert_eq!(gpl3.category.as_str(), "Strong Copyleft");

    let unlicense = find_license("unlicense").unwrap();
    assert_eq!(unlicense.category, LicenseCategory::PublicDomain);
    assert_eq!(unlicense.category.as_str(), "Public Domain");
}

#[test]
fn test_unknown_license_returns_none() {
    assert!(find_license("not-a-real-license").is_none());
    assert!(find_license("").is_none());
}
