# Keiros 

Keiros is a modular **red team agent framework** built in Rust, designed for use with custom Command & Control (C2) systems. Allowing operators to craft tailored malware or implants using a declarative profile system and Rust's powerful conditional compilation.

##  Features

- Modular architecture using Cargo features
- Linux and  Windows cross-compilation support via Docker
- Multiple communication handlers (HTTP, socket)
- Dynamic feature registration and initialization
- CLI tooling to speed up agent development and builds

##  Getting Started

###  Prerequisites

- [Docker](https://www.docker.com/)
- [Rust](https://www.rust-lang.org/)
- Git

##  Install

```bash
git clone https://github.com/your-org/keiros.git
cd keiros
cargo build --release
```

## Usage

###  Initialize a New Agent Project
```
./keiros init --project client --listener http
```
- ```--project```: Name of the new binary Cargo project (e.g., client)
-``` --listener```: Communication handler to include (http or socket)

### Create a New Feature
```
./keiros feature new --name execute
```
Creates src/features/execute.rs and registers it in mod.rs.

### Add Profile for Compilation
Edit or use built-in profiles in build_profiles/:

```
name: linux_agent
target: x86_64-unknown-linux-musl
release: true
strip: true
enabled_features:
  - register_agent
  - execute
  - http
```

### Build the Agent
```
./keiros build --profile linux_agent
```
