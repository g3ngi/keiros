use keiros::init::init_agent_structure;
use keiros::features::{create_feature, register_feature_in_mod_rs, list_registered_features};
use keiros::feature_loader::{generate_loader, read_feature_map, FeaturesMap, FeatureEntry};
use keiros::patcher::{patch_main_rs, ensure_mod_features_declared};
use keiros::compiler::compile;
use keiros::profile::BuildProfile;
use keiros::feature_map_gen::generate_feature_map;

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use std::process::Command;

#[derive(Parser)]
#[command(name = "keiros")]
#[command(about = "Keiros - Red Team Agent Arsenal", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        agent_name: Option<String>,

        #[arg(long, help = "Skip running `cargo new`, create structure only")]
        no_cargo: bool,
    },
    Feature {
        #[command(subcommand)]
        cmd: FeatureCommand,
    },
    Build {
        #[arg(short, long, default_value = "linux_agent")]
        profile: String,

        #[arg(short, long)]
        listener: Option<String>,
    },
    Clean {
        #[arg(short, long, help = "Also remove target_output directory")]
        all: bool,

        #[arg(long, help = "Only show what would be deleted")]
        dry_run: bool,

        #[arg(long, help = "Comma-separated list of image prefixes to keep")]
        keep: Option<String>,
    },
    Generate,
    Autofill,
}

#[derive(Subcommand)]
enum FeatureCommand {
    New {
        #[arg(short, long)]
        name: String,
    },
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { agent_name, no_cargo } => {
            init_agent_structure(agent_name.as_deref(), no_cargo)?;
        }

        Commands::Feature { cmd } => match cmd {
            FeatureCommand::New { name } => handle_new_feature(&name)?,
            FeatureCommand::List => {
                let features = list_registered_features()?;
                for feat in features {
                    println!("- {}", feat);
                }
            }
        },

        Commands::Build { profile, listener } => handle_build(&profile, listener)?,

        Commands::Clean { all, dry_run, keep } => handle_clean(all, dry_run, keep)?,

        Commands::Generate => {
            let feature_map = read_feature_map(Path::new("feature_map.yml"))?;
            generate_loader(&feature_map)?;
        }

        Commands::Autofill => {
            generate_feature_map()?;
        }
    }

    Ok(())
}

fn handle_new_feature(feature_name: &str) -> Result<()> {
    create_feature(feature_name)?;
    register_feature_in_mod_rs(feature_name)?;

    let all_features = list_registered_features()?;
    let feature_map = generate_feature_map_from_vec(&all_features);
    generate_loader(&feature_map)?;
    Ok(())
}

fn handle_build(profile_name: &str, listener: Option<String>) -> Result<()> {
    let profile_path = format!("build_profiles/{}.yml", profile_name);
    let mut profile = BuildProfile::from_file(Path::new(&profile_path))?;

    println!("[*] Loaded profile: {}", profile.name);

    if let Some(ref l) = listener {
        println!("[*] Overriding listener to: {}", l);
        profile.enabled_features.retain(|f| f != "http" && f != "socket");
        profile.enabled_features.push(l.clone());
    }

    let enabled = profile.enabled_features.clone();

    ensure_mod_features_declared()?;
    let feature_map = generate_feature_map_from_vec(&enabled);
    generate_loader(&feature_map)?;
    patch_main_rs()?;
    compile(&profile);
    Ok(())
}

fn handle_clean(all: bool, dry_run: bool, keep: Option<String>) -> Result<()> {
    println!("[*] Cleaning up keiros Docker images...");

    let keep_prefixes: Vec<String> = keep
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let output = Command::new("docker")
        .args(&["images", "--format", "{{.Repository}}"])
        .output()
        .with_context(|| "Failed to run `docker images`. Is Docker installed and in PATH?")?;

    let images = String::from_utf8_lossy(&output.stdout);
    let mut to_delete: Vec<String> = vec![];

    for image in images.lines() {
        if image.starts_with("keiros-") && !keep_prefixes.iter().any(|p| image.starts_with(p)) {
            to_delete.push(image.to_string());
        }
    }

    if dry_run {
        println!("[DRY RUN] The following Docker images would be removed:");
        for image in &to_delete {
            println!(" - {}", image);
        }
    } else {
        for image in &to_delete {
            println!("[*] Removing image: {}", image);
            let _ = Command::new("docker")
                .args(&["rmi", "-f", image])
                .status();
        }
    }

    if all {
        println!("[*] Removing ./target_output...");
        let output_path = Path::new("./target_output");
        if output_path.exists() {
            if dry_run {
                println!("[DRY RUN] Would remove ./target_output");
            } else {
                println!("[*] Removing ./target_output");
                fs::remove_dir_all(output_path)?;
            }
        }
    }

    println!("[+] Cleanup complete.");
    Ok(())
}

fn generate_feature_map_from_vec(features: &[String]) -> FeaturesMap {
    let map = features
        .iter()
        .map(|f| {
            (
                f.clone(), 
                FeatureEntry {
                    path: f.clone(),
                    call: Some("init".to_string()),
                },
            )
        })
        .collect();
    FeaturesMap(map)
}
