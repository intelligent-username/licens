use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    Normal,
    Dev,
    Build,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub kind: DependencyKind,
    pub source_manifest: String,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictSeverity {
    Safe,
    Warning,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConflict {
    pub dependency_name: String,
    pub dependency_license: String,
    pub project_license: String,
    pub severity: ConflictSeverity,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub project_license: Option<String>,
    pub total_dependencies: usize,
    pub audited_dependencies: usize,
    pub dependencies: Vec<Dependency>,
    pub conflicts: Vec<LicenseConflict>,
    pub recommended_permissive: String,
    pub recommended_restrictive: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Markdown,
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Markdown
    }
}
