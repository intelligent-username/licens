#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseCategory {
    PublicDomain,
    Permissive,
    CopyleftWeak,
    CopyleftStrong,
}

impl LicenseCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseCategory::PublicDomain => "Public Domain",
            LicenseCategory::Permissive => "Permissive",
            LicenseCategory::CopyleftWeak => "Weak Copyleft",
            LicenseCategory::CopyleftStrong => "Strong Copyleft",
        }
    }
}

pub struct LicenseTemplate {
    pub key: &'static str,
    pub spdx_id: &'static str,
    pub name: &'static str,
    pub category: LicenseCategory,
    pub text: &'static str,
}

impl LicenseTemplate {
    pub fn render(&self, year: i32, author: &str) -> String {
        let y_str = year.to_string();
        self.text
            .replace("[year]", &y_str)
            .replace("[Year]", &y_str)
            .replace("<YEAR>", &y_str)
            .replace("<year>", &y_str)
            .replace("[yyyy]", &y_str)
            .replace("[author]", author)
            .replace("[fullname]", author)
            .replace("<COPYRIGHT HOLDER>", author)
            .replace("<OWNER>", author)
            .replace("<name of author>", author)
            .replace("[Name of Organization]", author)
            .replace("[name of copyright owner]", author)
    }
}

pub static LICENSES: &[LicenseTemplate] = &[
    LicenseTemplate {
        key: "mit",
        spdx_id: "MIT",
        name: "MIT License",
        category: LicenseCategory::Permissive,
        text: include_str!("content/mit.txt"),
    },
    LicenseTemplate {
        key: "apache-2.0",
        spdx_id: "Apache-2.0",
        name: "Apache License 2.0",
        category: LicenseCategory::Permissive,
        text: include_str!("content/apache-2.0.txt"),
    },
    LicenseTemplate {
        key: "gpl-2.0",
        spdx_id: "GPL-2.0",
        name: "GNU General Public License v2.0",
        category: LicenseCategory::CopyleftStrong,
        text: include_str!("content/gpl-2.0.txt"),
    },
    LicenseTemplate {
        key: "gpl-3.0",
        spdx_id: "GPL-3.0",
        name: "GNU General Public License v3.0",
        category: LicenseCategory::CopyleftStrong,
        text: include_str!("content/gpl-3.0.txt"),
    },
    LicenseTemplate {
        key: "agpl-3.0",
        spdx_id: "AGPL-3.0",
        name: "GNU Affero General Public License v3.0",
        category: LicenseCategory::CopyleftStrong,
        text: include_str!("content/agpl-3.0.txt"),
    },
    LicenseTemplate {
        key: "lgpl-2.1",
        spdx_id: "LGPL-2.1",
        name: "GNU Lesser General Public License v2.1",
        category: LicenseCategory::CopyleftWeak,
        text: include_str!("content/lgpl-2.1.txt"),
    },
    LicenseTemplate {
        key: "mpl-2.0",
        spdx_id: "MPL-2.0",
        name: "Mozilla Public License 2.0",
        category: LicenseCategory::CopyleftWeak,
        text: include_str!("content/mpl-2.0.txt"),
    },
    LicenseTemplate {
        key: "bsd-1-clause",
        spdx_id: "BSD-1-Clause",
        name: "BSD 1-Clause License",
        category: LicenseCategory::Permissive,
        text: include_str!("content/bsd-1-clause.txt"),
    },
    LicenseTemplate {
        key: "bsd-2-clause",
        spdx_id: "BSD-2-Clause",
        name: "BSD 2-Clause License",
        category: LicenseCategory::Permissive,
        text: include_str!("content/bsd-2-clause.txt"),
    },
    LicenseTemplate {
        key: "bsd-3-clause",
        spdx_id: "BSD-3-Clause",
        name: "BSD 3-Clause License",
        category: LicenseCategory::Permissive,
        text: include_str!("content/bsd-3-clause.txt"),
    },
    LicenseTemplate {
        key: "bsl-1.0",
        spdx_id: "BSL-1.0",
        name: "Boost Software License 1.0",
        category: LicenseCategory::Permissive,
        text: include_str!("content/bsl-1.0.txt"),
    },
    LicenseTemplate {
        key: "epl-2.0",
        spdx_id: "EPL-2.0",
        name: "Eclipse Public License 2.0",
        category: LicenseCategory::CopyleftWeak,
        text: include_str!("content/epl-2.0.txt"),
    },
    LicenseTemplate {
        key: "isc",
        spdx_id: "ISC",
        name: "ISC License",
        category: LicenseCategory::Permissive,
        text: include_str!("content/isc.txt"),
    },
    LicenseTemplate {
        key: "cc0-1.0",
        spdx_id: "CC0-1.0",
        name: "Creative Commons Zero v1.0 Universal",
        category: LicenseCategory::PublicDomain,
        text: include_str!("content/cc0-1.0.txt"),
    },
    LicenseTemplate {
        key: "unlicense",
        spdx_id: "Unlicense",
        name: "The Unlicense",
        category: LicenseCategory::PublicDomain,
        text: include_str!("content/unlicense.txt"),
    },
];

pub fn find_license(query: &str) -> Option<&'static LicenseTemplate> {
    let q = query.trim().to_lowercase();
    LICENSES.iter().find(|l| {
        l.key.to_lowercase() == q
            || l.spdx_id.to_lowercase() == q
            || l.name.to_lowercase() == q
    })
}
