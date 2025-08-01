use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

pub fn init_agent_structure(dir: Option<&str>, no_cargo: bool) -> Result<()> {
    let dir = dir.unwrap_or("client");

    if !no_cargo {
        println!("[*] Creating new Keiros agent at `{}` using `cargo new`...", dir);
        let status = Command::new("cargo")
            .args(["new", dir, "--bin"])
            .status()
            .context("Failed to run `cargo new`.")?;

        if !status.success(){
            return Err(anyhow::anyhow!("`cargo new` failed with with status {:?}", status));
        }
    } else {
        println!("[*] Creating new Keiros agent at `{}` (no cargo)...", dir);
        fs::create_dir_all(format!("{}/src", dir))?;
    }

    let features_dir = format!("{}/src/features", dir);
    let handlers_dir = format!("{}/src/comms", dir);
    fs::create_dir_all(&features_dir)?;
    fs::create_dir_all(&handlers_dir)?;

    // Create main.rs skeleton
    let main_rs = r#"
mod agent;
mod features;

fn main() {
    println!("[*] Keiros agent starting...");
    agent::run();
}
"#;

    let agent_rs = r#"
pub fn run() {
    println!("[*] Agent running...");
    // TODO: Feature loader and communication loop
}
"#;

    let features_mod_rs = r#"
pub fn load_features() {
    // Dynamically loaded features will go here
}
"#;

    let comms_mod_rs = r#" 
// add your communication channel identifier here
// #[cfg[feature = "comms_channel"]]
// pub mod comms_channel

use std::error::Error;

pub fn trait CommsChannel(){
    fn register_agent(&mut self) -> Result<String, Box<dyn Error>>;
    fn poll_command(&mut self) -> Result<String, Box<dyn Error>>;
    fn report_result(&mut self, result: &str) -> Result<(), Box<dyn Error>>;   
}
"#;

    let types_rs = r#"
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentInfo {
    pub hostname: String,
    pub os: String,
    pub ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    pub agent_id: String,
    pub status: String,
}
"#;

    write_file(format!("{}/src/main.rs", dir), main_rs)?;
    write_file(format!("{}/src/agent.rs", dir), agent_rs)?;
    write_file(format!("{}/src/features/mod.rs", dir), features_mod_rs)?;
    write_file(format!("{}/src/comms/mod.rs", dir), comms_mod_rs)?;
    write_file(format!("{}/src/types.rs", dir), types_rs);

    println!("[*] Creating build_profiles...");
    let profile_dir = format!("{}/build_profiles", dir);
    fs::create_dir_all(&profile_dir)?;

    let linux_profile = r#"
name: linux_agent
target: x86_64-unknown-linux-musl
release: true
strip: true
enabled_features:
  - register_agent
  - execute_command
  - report_result
"#;

    let windows_profile = r#"
name: windows_agent
target: x86_64-pc-windows-gnu
release: true
strip: true
enabled_features:
  - register_agent
  - execute_command
  - report_result
"#;

    write_file(format!("{}/linux_agent.yml", profile_dir), linux_profile)?;
    write_file(format!("{}/windows_agent.yml", profile_dir), windows_profile)?;

    let feature_map = r#"

"#;
    write_file(format!("{}/feature_map.yml", dir), feature_map)?;

let dockerfile = r#"
# syntax=docker/dockerfile:1.4
FROM ubuntu:24.04

LABEL maintainer="gengi"

ARG PROFILE=linux_default
ARG TARGET=x86_64-unknown-linux-musl
ARG RELEASE=false
ARG FEATURES=
ARG STRIP=false
ARG SERVER_IP=127.0.0.1
ARG LISTENER_PORT=8080

ENV PROFILE=${PROFILE}
ENV TARGET=${TARGET}
ENV RELEASE=${RELEASE}
ENV FEATURES=${FEATURES}
ENV STRIP=${STRIP}
ENV SERVER_IP=${SERVER_IP}
ENV LISTENER_PORT=${LISTENER_PORT}

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl git pkg-config libssl-dev ca-certificates \
    file sudo g++-mingw-w64-x86-64 g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross musl-tools && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup target add \
    x86_64-unknown-linux-musl \
    aarch64-unknown-linux-gnu \
    x86_64-pc-windows-gnu

COPY ./build_profiles /app/build_profiles
COPY ./src /app/src/
COPY ./Cargo.toml /app

CMD bash -c '\
    set -e; \
    echo "[*] SERVER_IP=$SERVER_IP LISTENER_PORT=$LISTENER_PORT"; \
    BUILD_CMD="cargo build --target $TARGET"; \
    if [ "$RELEASE" = "true" ]; then BUILD_CMD="$BUILD_CMD --release"; fi; \
    if [ -n "$FEATURES" ]; then BUILD_CMD="$BUILD_CMD --features $FEATURES"; fi; \
    echo "[*] Running build: $BUILD_CMD"; \
    eval $BUILD_CMD; \
    BIN_DIR="target/$TARGET/$( [ "$RELEASE" = "true" ] && echo release || echo debug )"; \
    mkdir -p /output; \
    for bin in $BIN_DIR/*; do \
        if [ -x "$bin" ] && [ -f "$bin" ]; then \
            [ "$STRIP" = "true" ] && echo "[*] Stripping $bin" && strip "$bin"; \
            echo "[*] Copying $bin to /output"; \
            cp "$bin" /output/; \
        fi; \
    done; \
    echo "[+] Done." \
'
"#;
    write_file(format!("{}/Dockerfile", dir), dockerfile);

    println!("[+] Keiros agent skeleton created in `{}`.", dir);
    Ok(())
}

fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let mut file = File::create(path.as_ref())
        .with_context(|| format!("Failed to create {}", path.as_ref().display()))?;
    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to {}", path.as_ref().display()))?;
    Ok(())
}
