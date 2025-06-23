use serde::Deserialize;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize)]
pub struct BuildProfile {
    pub name: String,
    pub target: Option<String>,
    pub release: Option<bool>,
    pub strip: Option<bool>,
    pub enabled_features: Vec<String>,
}

impl BuildProfile {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read profile file at {}", path.display()))?;
        let profile: Self = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML from {}", path.display()))?;
        Ok(profile)
    }
}

