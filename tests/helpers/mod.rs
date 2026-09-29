use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use licens::audit::model::{Dependency, DependencyKind};

pub struct TestWorkspace {
    dir: TempDir,
}

impl TestWorkspace {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().expect("Failed to create temporary test workspace"),
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write_file(&self, relative_path: impl AsRef<Path>, content: &str) -> PathBuf {
        let full_path = self.dir.path().join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        fs::write(&full_path, content).expect("Failed to write mock test file");
        full_path
    }

    pub fn write_cargo_toml(&self, content: &str) -> PathBuf {
        self.write_file("Cargo.toml", content)
    }

    pub fn write_package_json(&self, content: &str) -> PathBuf {
        self.write_file("package.json", content)
    }

    pub fn write_pyproject_toml(&self, content: &str) -> PathBuf {
        self.write_file("pyproject.toml", content)
    }

    pub fn write_git_config(&self, content: &str) -> PathBuf {
        self.write_file(".git/config", content)
    }
}

pub fn make_dependency(
    name: &str,
    version: Option<&str>,
    license: Option<&str>,
    kind: DependencyKind,
    manifest: &str,
) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: version.map(|s| s.to_string()),
        kind,
        source_manifest: manifest.to_string(),
        license: license.map(|s| s.to_string()),
    }
}
