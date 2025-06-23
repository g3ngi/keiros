use std::fs;
use std::path::Path;
use regex::Regex;
use anyhow::{Result, anyhow};

pub fn patch_main_rs() -> Result<()> {
    let main_path = Path::new("src/main.rs");
    let content = fs::read_to_string(main_path)?;
    
    if content.contains("init_features();") {
        println!("[!] main.rs is already patched.");
        return Ok(());
    }

    let re_main = Regex::new(r"(fn\s+main\s*\(\s*\)\s*\{)")?;
    let patch_import = "mod feature_loader;\nuse feature_loader::init_features;\n";

    if let Some(main_match) = re_main.find(&content) {
        let (head, tail) = content.split_at(main_match.start());

        let new_head = if head.contains("use feature_loader::init_features;") {
            head.to_string()
        } else {
            format!("{head}\n// AUTO-GENERATED\n{patch_import}")
        };

        let main_body_start = main_match.as_str();
        let new_main = format!("{new_head}{main_body_start}\n    init_features();{}", &tail[main_match.end()..]);

        fs::write(main_path, new_main);
        println!("[!] main.rs sucessfully patched");
        Ok(())
    } else {
        Err(anyhow!("main() function not found in src/main.rs"))
    }
}

pub fn ensure_mod_features_declared() -> Result<()> {
    let main_path = Path::new("src/main.rs");
    let content = fs::read_to_string(main_path)?;

    if content.contains("mod features;"){
        return Ok(())
    }

    let mut lines: Vec<&str> = content.lines().collect();
    let insert_index = lines.iter()
        .position(|line| line.trim().starts_with("fn main"))
        .unwrap_or(0);
    lines.insert(insert_index, "mod features;");

    fs::write(main_path, lines.join("\n"))?;
    println!("[+] Injected `mod features;` into main.rs");
    Ok(())
}