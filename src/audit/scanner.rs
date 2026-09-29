use std::fs;
use std::path::Path;

use crate::audit::model::{Dependency, DependencyKind};
use crate::error::{LicensError, Result};

pub struct ScannedProject {
    pub detected_license: Option<String>,
    pub dependencies: Vec<Dependency>,
}

pub struct ManifestScanner;

impl ManifestScanner {
    pub fn scan_current_dir(include_dev: bool, include_build: bool) -> Result<ScannedProject> {
        let mut all_deps = Vec::new();
        let mut detected_license = None;
        let mut found_any = false;

        // 1. Cargo.toml
        if Path::new("Cargo.toml").exists() {
            found_any = true;
            let (lic, deps) = Self::scan_cargo_toml(include_dev, include_build)?;
            if detected_license.is_none() {
                detected_license = lic;
            }
            all_deps.extend(deps);
        }

        // 2. package.json
        if Path::new("package.json").exists() {
            found_any = true;
            let (lic, deps) = Self::scan_package_json(include_dev)?;
            if detected_license.is_none() {
                detected_license = lic;
            }
            all_deps.extend(deps);
        }

        // 3. pyproject.toml
        if Path::new("pyproject.toml").exists() {
            found_any = true;
            let (lic, deps) = Self::scan_pyproject_toml(include_dev)?;
            if detected_license.is_none() {
                detected_license = lic;
            }
            all_deps.extend(deps);
        }

        if !found_any {
            return Err(LicensError::ManifestNotFound);
        }

        Ok(ScannedProject {
            detected_license,
            dependencies: all_deps,
        })
    }

    fn scan_cargo_toml(
        include_dev: bool,
        include_build: bool,
    ) -> Result<(Option<String>, Vec<Dependency>)> {
        let content = fs::read_to_string("Cargo.toml")?;
        let table: toml::Table = toml::from_str(&content)?;

        let detected_license = table
            .get("package")
            .and_then(|p| p.get("license"))
            .or_else(|| {
                table
                    .get("workspace")
                    .and_then(|w| w.get("package"))
                    .and_then(|p| p.get("license"))
            })
            .and_then(|l| l.as_str())
            .map(|s| s.to_string());

        let mut deps = Vec::new();

        // [dependencies]
        if let Some(dep_table) = table.get("dependencies").and_then(|d| d.as_table()) {
            for (name, val) in dep_table {
                let version = Self::extract_toml_version(val);
                deps.push(Dependency {
                    name: name.clone(),
                    version,
                    kind: DependencyKind::Normal,
                    source_manifest: "Cargo.toml".to_string(),
                    license: None,
                });
            }
        }

        // [workspace.dependencies] fallback if root has no package dependencies
        if deps.is_empty() {
            if let Some(dep_table) = table
                .get("workspace")
                .and_then(|w| w.get("dependencies"))
                .and_then(|d| d.as_table())
            {
                for (name, val) in dep_table {
                    let version = Self::extract_toml_version(val);
                    deps.push(Dependency {
                        name: name.clone(),
                        version,
                        kind: DependencyKind::Normal,
                        source_manifest: "Cargo.toml (workspace)".to_string(),
                        license: None,
                    });
                }
            }
        }

        // [dev-dependencies]
        if include_dev {
            if let Some(dep_table) = table.get("dev-dependencies").and_then(|d| d.as_table()) {
                for (name, val) in dep_table {
                    let version = Self::extract_toml_version(val);
                    deps.push(Dependency {
                        name: name.clone(),
                        version,
                        kind: DependencyKind::Dev,
                        source_manifest: "Cargo.toml".to_string(),
                        license: None,
                    });
                }
            }
        }

        // [build-dependencies]
        if include_build {
            if let Some(dep_table) = table.get("build-dependencies").and_then(|d| d.as_table()) {
                for (name, val) in dep_table {
                    let version = Self::extract_toml_version(val);
                    deps.push(Dependency {
                        name: name.clone(),
                        version,
                        kind: DependencyKind::Build,
                        source_manifest: "Cargo.toml".to_string(),
                        license: None,
                    });
                }
            }
        }

        Ok((detected_license, deps))
    }

    fn scan_package_json(include_dev: bool) -> Result<(Option<String>, Vec<Dependency>)> {
        let content = fs::read_to_string("package.json")?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let detected_license = json
            .get("license")
            .and_then(|l| {
                l.as_str().map(|s| s.to_string()).or_else(|| {
                    l.get("type").and_then(|t| t.as_str()).map(|s| s.to_string())
                })
            });

        let mut deps = Vec::new();

        if let Some(dep_obj) = json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, val) in dep_obj {
                deps.push(Dependency {
                    name: name.clone(),
                    version: val.as_str().map(|s| s.to_string()),
                    kind: DependencyKind::Normal,
                    source_manifest: "package.json".to_string(),
                    license: None,
                });
            }
        }

        if include_dev {
            if let Some(dep_obj) = json.get("devDependencies").and_then(|d| d.as_object()) {
                for (name, val) in dep_obj {
                    deps.push(Dependency {
                        name: name.clone(),
                        version: val.as_str().map(|s| s.to_string()),
                        kind: DependencyKind::Dev,
                        source_manifest: "package.json".to_string(),
                        license: None,
                    });
                }
            }
        }

        Ok((detected_license, deps))
    }

    fn scan_pyproject_toml(include_dev: bool) -> Result<(Option<String>, Vec<Dependency>)> {
        let content = fs::read_to_string("pyproject.toml")?;
        let table: toml::Table = toml::from_str(&content)?;

        let mut detected_license = None;

        // PEP 621 [project.license]
        if let Some(project) = table.get("project") {
            if let Some(lic) = project.get("license") {
                if let Some(s) = lic.as_str() {
                    detected_license = Some(s.to_string());
                } else if let Some(t) = lic.get("text").and_then(|v| v.as_str()) {
                    detected_license = Some(t.to_string());
                }
            }
        }

        // Poetry [tool.poetry.license]
        if detected_license.is_none() {
            if let Some(poetry) = table.get("tool").and_then(|t| t.get("poetry")) {
                if let Some(s) = poetry.get("license").and_then(|l| l.as_str()) {
                    detected_license = Some(s.to_string());
                }
            }
        }

        let mut deps = Vec::new();

        // PEP 621 [project.dependencies]
        if let Some(project) = table.get("project") {
            if let Some(dep_arr) = project.get("dependencies").and_then(|d| d.as_array()) {
                for val in dep_arr {
                    if let Some(spec) = val.as_str() {
                        let (name, ver) = Self::parse_python_dependency_spec(spec);
                        deps.push(Dependency {
                            name,
                            version: ver,
                            kind: DependencyKind::Normal,
                            source_manifest: "pyproject.toml".to_string(),
                            license: None,
                        });
                    }
                }
            }

            if include_dev {
                if let Some(opt_deps) = project.get("optional-dependencies").and_then(|d| d.as_table()) {
                    for (_group, list) in opt_deps {
                        if let Some(arr) = list.as_array() {
                            for val in arr {
                                if let Some(spec) = val.as_str() {
                                    let (name, ver) = Self::parse_python_dependency_spec(spec);
                                    deps.push(Dependency {
                                        name,
                                        version: ver,
                                        kind: DependencyKind::Dev,
                                        source_manifest: "pyproject.toml".to_string(),
                                        license: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Poetry [tool.poetry.dependencies]
        if let Some(poetry) = table.get("tool").and_then(|t| t.get("poetry")) {
            if let Some(p_deps) = poetry.get("dependencies").and_then(|d| d.as_table()) {
                for (name, val) in p_deps {
                    if name.eq_ignore_ascii_case("python") {
                        continue;
                    }
                    let ver = Self::extract_toml_version(val);
                    deps.push(Dependency {
                        name: name.clone(),
                        version: ver,
                        kind: DependencyKind::Normal,
                        source_manifest: "pyproject.toml (poetry)".to_string(),
                        license: None,
                    });
                }
            }

            if include_dev {
                if let Some(p_dev) = poetry.get("group").and_then(|g| g.get("dev")).and_then(|d| d.get("dependencies")).and_then(|d| d.as_table()) {
                    for (name, val) in p_dev {
                        let ver = Self::extract_toml_version(val);
                        deps.push(Dependency {
                            name: name.clone(),
                            version: ver,
                            kind: DependencyKind::Dev,
                            source_manifest: "pyproject.toml (poetry-dev)".to_string(),
                            license: None,
                        });
                    }
                }
            }
        }

        Ok((detected_license, deps))
    }

    fn extract_toml_version(val: &toml::Value) -> Option<String> {
        match val {
            toml::Value::String(s) => Some(s.clone()),
            toml::Value::Table(t) => t.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()),
            _ => None,
        }
    }

    fn parse_python_dependency_spec(spec: &str) -> (String, Option<String>) {
        let op_indices = [">=", "<=", "==", "!=", "~=", ">", "<"]
            .iter()
            .filter_map(|op| spec.find(op).map(|idx| (idx, *op)))
            .collect::<Vec<_>>();

        if let Some(&(idx, _op)) = op_indices.iter().min_by_key(|(idx, _)| *idx) {
            let name = spec[..idx].trim().to_string();
            let version = spec[idx..].trim().to_string();
            (name, Some(version))
        } else {
            (spec.trim().to_string(), None)
        }
    }
}
