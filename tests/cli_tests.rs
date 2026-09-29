use licens::templates::{find_license, LICENSES};

#[test]
fn test_all_supported_licenses_exist() {
    let expected = [
        "mit",
        "gpl-2.0",
        "gpl-3.0",
        "apache-2.0",
        "bsd-1-clause",
        "bsd-2-clause",
        "bsd-3-clause",
        "isc",
        "unlicense",
        "wtfpl",
    ];

    for key in expected {
        assert!(
            find_license(key).is_some(),
            "Expected license '{}' was not found in catalog",
            key
        );
    }
}

#[test]
fn test_license_rendering() {
    let mit = find_license("mit").expect("MIT license should exist");
    let rendered = mit.render(2026, "Jane Doe <jane@example.com>");

    assert!(rendered.contains("2026"));
    assert!(rendered.contains("Jane Doe <jane@example.com>"));
    assert!(rendered.contains("Permission is hereby granted"));
}
