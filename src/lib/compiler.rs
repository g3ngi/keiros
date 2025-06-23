use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use crate::profile::BuildProfile;

pub fn compile(profile: &BuildProfile) -> bool {
    if !check_docker_running() {
        eprintln!("[x] Docker daemon is not running. Please start Docker and try again.");
        return false;
    }

    ensure_docker_access();

    let profile_name = profile.name.trim();
    let image_name = format!("keiros-builder");

    let target = profile
        .target
        .clone()
        .unwrap_or_else(|| "x86_64-unknown-linux-musl".to_string());
    let release = profile.release.unwrap_or(false).to_string();
    let strip = profile.strip.unwrap_or(false).to_string();
    let features = profile.enabled_features.join(",");

    if !check_image_exists(&image_name) {
        println!("[*] Docker image `{}` not found. Building...", image_name);
        let build_status = Command::new("docker")
            .args(&[
                "build",
                "--build-arg",
                &format!("PROFILE={}", profile_name),
                "--build-arg",
                &format!("TARGET={}", target),
                "--build-arg",
                &format!("RELEASE={}", release),
                "--build-arg",
                &format!("STRIP={}", strip),
                "--build-arg",
                &format!("FEATURES={}", features),
                "-t",
                &image_name,
                ".",
            ])
            .status()
            .expect("Failed to run docker build");

        if !build_status.success() {
            eprintln!("[x] Docker image build failed.");
            return false;
        }
    } else {
        println!("[*] Using cached Docker image `{}`", image_name);
    }

    let output_path = Path::new("./target_output");
    if !output_path.exists() {
        fs::create_dir_all(output_path).expect("Failed to create target_output directory");
    }

    println!("[*] Running Docker container to compile binary...");
    let run_status = Command::new("docker")
        .args(&[
            "run",
            "--rm",
            "-v",
            &format!("{}:/output", output_path.canonicalize().unwrap().display()),
            &image_name,
        ])
        .status()
        .expect("Failed to run docker container");

    if run_status.success() {
        println!("[+] Build complete. Output binary in ./target_output");
        true
    } else {
        eprintln!("[x] Docker run failed.");
        false
    }
}

fn check_docker_running() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn ensure_docker_access() {
    if !check_docker_running() {
        eprintln!("[!] Docker not accessible. Try adding your user to the docker group:");
        eprintln!("    sudo usermod -aG docker $USER");
        eprintln!("Then log out and back in, or run `newgrp docker`.");
        std::process::exit(1);
    }
}

fn check_image_exists(image: &str) -> bool {
    Command::new("docker")
        .args(&["images", "-q", image])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}
