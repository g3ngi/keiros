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
./keiros init --agent-name "IcyBear"
```
- ```--agent-name```: Name of the new binary Cargo project (e.g., client)

### Create a New Feature
```
./keiros feature new --name "pivot module"
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
./keiros build --profile linux_agent --teamserver-ip <your_c2_ip> --listener-port <your_c2_Lport>

```
- ```--profile```: build configuration for the binary
- ```--teamserver-ip```: your C2 ip address for comms
- ```--listener-port```: your C2 listener port for comms