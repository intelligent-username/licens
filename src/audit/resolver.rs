use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::audit::model::Dependency;
use crate::error::Result;

#[derive(Deserialize)]
struct CratesIoResponse {
    versions: Option<Vec<CratesIoVersion>>,
}

#[derive(Deserialize)]
struct CratesIoVersion {
    num: String,
    license: Option<String>,
}

pub struct LicenseResolver {
    client: reqwest::Client,
}

impl LicenseResolver {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("licens-cli-tool/0.1.0 (github.com/project/licens)"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    pub async fn resolve_all(&self, mut dependencies: Vec<Dependency>) -> Vec<Dependency> {
        let lock_map = Self::try_read_cargo_lock();

        for dep in &mut dependencies {
            if dep.source_manifest == "Cargo.toml" {
                if let Some(ref lock) = lock_map {
                    if let Some(lic) = lock.get(&dep.name) {
                        dep.license = Some(lic.clone());
                        continue;
                    }
                }

                // Query crates.io if not found in lockfile
                if dep.license.is_none() {
                    if let Ok(lic) = self.fetch_crates_io_license(&dep.name, dep.version.as_deref()).await {
                        dep.license = Some(lic);
                    }
                }
            } else if dep.source_manifest == "package.json" {
                // If local node_modules exists, attempt to read package.json
                let node_pkg = Path::new("node_modules").join(&dep.name).join("package.json");
                if node_pkg.exists() {
                    if let Ok(c) = fs::read_to_string(node_pkg) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&c) {
                            if let Some(l) = json.get("license").and_then(|v| v.as_str()) {
                                dep.license = Some(l.to_string());
                            }
                        }
                    }
                }
            }

            if dep.license.is_none() {
                dep.license = Some("Unknown".to_string());
            }
        }

        dependencies
    }

    async fn fetch_crates_io_license(
        &self,
        crate_name: &str,
        target_version: Option<&str>,
    ) -> Result<String> {
        let url = format!("https://crates.io/api/v1/crates/{crate_name}");
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let data: CratesIoResponse = resp.json().await?;

        if let Some(versions) = data.versions {
            if let Some(ver) = target_version {
                let clean_ver = ver.trim_start_matches(['^', '~', '=', 'v']);
                if let Some(matched) = versions.iter().find(|v| v.num == clean_ver) {
                    if let Some(ref lic) = matched.license {
                        return Ok(lic.clone());
                    }
                }
            }

            if let Some(first) = versions.first() {
                if let Some(ref lic) = first.license {
                    return Ok(lic.clone());
                }
            }
        }

        Ok("Unknown".to_string())
    }

    fn try_read_cargo_lock() -> Option<std::collections::HashMap<String, String>> {
        let path = Path::new("Cargo.lock");
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let table: toml::Table = toml::from_str(&content).ok()?;
        let packages = table.get("package")?.as_array()?;

        let mut map = std::collections::HashMap::new();
        for p in packages {
            if let (Some(name), Some(lic)) = (
                p.get("name").and_then(|v| v.as_str()),
                p.get("license").and_then(|v| v.as_str()),
            ) {
                map.insert(name.to_string(), lic.to_string());
            }
        }
        Some(map)
    }
}
