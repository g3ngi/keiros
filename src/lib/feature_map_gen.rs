use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct FeatureEntry {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    call: Option<String>,
}

pub fn generate_feature_map() -> Result<()> {
    let mut features = HashMap::new();
    let base = Path::new("src");

    for entry in walkdir::WalkDir::new(base)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let file_path = entry.path();
        let rel_path = file_path.strip_prefix(base)?.with_extension("");

        let parts: Vec<String> = rel_path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        let path_str = parts.join("::");
        let feature_name = parts.last().unwrap().to_string();

        let content = fs::read_to_string(file_path)?;
        let has_init = content.contains("pub fn init");

        features.insert(
            feature_name.clone(),
            FeatureEntry {
                path: path_str,
                call: has_init.then_some("init()".to_string()),
            },
        );
    }

    let out = File::create("feature_map.yml")?;
    serde_yaml::to_writer(out, &features)?;
    println!("[+] Auto-filled feature_map.yml with {} entries", features.len());
    Ok(())
}