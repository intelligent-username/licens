use crate::audit::model::{ConflictSeverity, Dependency, LicenseConflict};
use crate::templates::find_license;
use crate::templates::licenses::LicenseCategory;

pub struct CompatibilityAnalyzer;

impl CompatibilityAnalyzer {
    pub fn analyze(
        project_license_name: Option<&str>,
        dependencies: &[Dependency],
    ) -> (Vec<LicenseConflict>, String, String) {
        let mut conflicts = Vec::new();

        let proj_template = project_license_name.and_then(find_license);

        let mut has_strong_copyleft = false;
        let mut has_weak_copyleft = false;
        let mut has_gpl2 = false;

        for dep in dependencies {
            let dep_lic_str = dep.license.as_deref().unwrap_or("Unknown");
            let dep_template = find_license(dep_lic_str);

            if let Some(template) = dep_template {
                match template.category {
                    LicenseCategory::CopyleftStrong => {
                        has_strong_copyleft = true;
                        if template.key == "gpl-2.0" {
                            has_gpl2 = true;
                        }
                    }
                    LicenseCategory::CopyleftWeak => {
                        has_weak_copyleft = true;
                    }
                    _ => {}
                }
            } else {
                let lower = dep_lic_str.to_lowercase();
                if lower.contains("gpl-3") || lower.contains("agpl") {
                    has_strong_copyleft = true;
                } else if lower.contains("gpl-2") {
                    has_strong_copyleft = true;
                    has_gpl2 = true;
                } else if lower.contains("lgpl") || lower.contains("mpl") {
                    has_weak_copyleft = true;
                }
            }

            if let Some(proj) = proj_template {
                if let Some(conflict) = Self::check_conflict(proj.key, dep.name.as_str(), dep_lic_str) {
                    conflicts.push(conflict);
                }
            }
        }

        // Determine recommended licenses
        let (rec_permissive, rec_restrictive) = if has_strong_copyleft {
            if has_gpl2 {
                ("GPL-2.0".to_string(), "GPL-2.0".to_string())
            } else {
                ("GPL-3.0".to_string(), "GPL-3.0".to_string())
            }
        } else if has_weak_copyleft {
            ("MIT (with dynamic linking)".to_string(), "GPL-3.0".to_string())
        } else {
            ("MIT".to_string(), "GPL-3.0".to_string())
        };

        (conflicts, rec_permissive, rec_restrictive)
    }

    fn check_conflict(
        proj_key: &str,
        dep_name: &str,
        dep_license: &str,
    ) -> Option<LicenseConflict> {
        let dep_lower = dep_license.to_lowercase();

        // If project is permissive (MIT, Apache, BSD, ISC) but dependency is Strong Copyleft (GPL)
        let is_proj_permissive = matches!(
            proj_key,
            "mit" | "apache-2.0" | "bsd-1-clause" | "bsd-2-clause" | "bsd-3-clause" | "isc" | "unlicense" | "wtfpl"
        );

        if is_proj_permissive && (dep_lower.contains("gpl") && !dep_lower.contains("lgpl")) {
            return Some(LicenseConflict {
                dependency_name: dep_name.to_string(),
                dependency_license: dep_license.to_string(),
                project_license: proj_key.to_uppercase(),
                severity: ConflictSeverity::Conflict,
                reason: format!(
                    "Dependency '{dep_name}' uses strong copyleft ({dep_license}), which requires downstream projects to be distributed under a compatible GPL license."
                ),
            });
        }

        // GPL-2.0 projects cannot use Apache-2.0 dependencies due to patent clause incompatibilities
        if proj_key == "gpl-2.0" && dep_lower.contains("apache-2.0") {
            return Some(LicenseConflict {
                dependency_name: dep_name.to_string(),
                dependency_license: dep_license.to_string(),
                project_license: "GPL-2.0".to_string(),
                severity: ConflictSeverity::Conflict,
                reason: format!(
                    "Apache-2.0 licenses contain patent clauses that are legally incompatible with GPL-2.0."
                ),
            });
        }

        None
    }
}
