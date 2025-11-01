//! Test Environment Management
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub config_path: Option<PathBuf>,
}

impl TestEnv {
    pub fn new() -> Self {
        TestEnv {
            temp_dir: TempDir::new().expect("Failed to create temp directory"),
            config_path: None,
        }
    }

    pub fn create_config(&mut self, content: &str) -> PathBuf {
        let path = self.temp_dir.path().join("raibid.yaml");
        std::fs::write(&path, content).expect("Failed to write config");
        self.config_path = Some(path.clone());
        path
    }

    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}
