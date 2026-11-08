use std::path::Path;
use tempfile::TempDir;

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
}
