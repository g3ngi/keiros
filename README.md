# Keiros

Keiros is a modular **red team agent framework** built in Rust, designed for use with custom Command & Control (C2) systems. It lets operators craft tailored implants using a **declarative build profile** and Rust’s **conditional compilation**, aiming to **simplify agent development** while maintaining **operational flexibility**, **portability**, and **security-by-design**.

> ⚠️ **Disclaimer (Legal & Ethical Use Only)**  
> Keiros is intended **solely** for authorized security testing, R&D, and education in environments where you have explicit permission. Using this tool against systems you do not own or administer may violate laws and agreements. **You are responsible** for complying with all applicable regulations and contracts.

> 🔧 **Migration Notice**  
> Keiros is currently undergoing a **major update**: migrating from the older *feature-based* design to a modern **module-based architecture**. Some tooling (CLI, builder logic, Docker workflows) may still reference legacy terms like `features`. The refactor will standardize on a full module system for cleaner, safer extensibility.

---

## Why Keiros?

- **Rust-first safety & performance**  
  Memory-safety by default, strong type system, and zero-cost abstractions reduce entire classes of defects while keeping runtime overhead low.

- **Composable, profile-driven builds**  
  A **declarative profile** enables repeatable, auditable builds: toggle modules, targets, and optimizations without hand-editing scattered configs.

- **Module-based architecture (next-gen)**  
  Clear boundaries between **comms**, **tasks**, and **capabilities** make it easier to extend, test, or replace components without forking the world.

- **Cross-platform, cross-toolchain**  
  Portable Rust + Dockerized cross-compilation (e.g., musl) helps target Linux and Windows from a consistent, reproducible build environment.

- **Operational ergonomics**  
  Opinionated CLI for common workflows (init, build, profile selection), minimizing glue code and shortening the loop from idea → implant.

- **Security-by-design defaults**  
  Minimal dependencies, feature-gated logging/telemetry, and explicit opt-in for risky functionality help avoid accidental operational exposure.

- **Teamserver-agnostic**  
  Keiros focuses on the **agent** layer. You can integrate it with your existing C2 backends and pipelines without being locked into a single stack.

---

## What is an Agent?

In Keiros, an **agent** is an (authorized) adversary-simulation component that runs on a target host and communicates with your C2. Conceptually, it is a **malicious program used for legitimate testing** with the following core behaviors:

- **Self-registration**  
  On first contact, the agent registers its **ID**, **IP**, **hostname**, **status**, and **communication protocol** with the C2.
- **Command polling**  
  It periodically **polls** for tasks (or receives them, depending on comms strategy).
- **Task execution**  
  It **executes** assigned tasks (capabilities provided by enabled modules) and **reports results** back to the C2.

> Note: The exact data model and cadence depend on your enabled modules and profile (e.g., HTTP vs. socket, beacon intervals, jitter, etc.).

---

## Features

- Modular architecture using Cargo features (transitioning to a full **module-based** system)
- Linux and Windows cross-compilation support via Docker
- Multiple communication handlers (HTTP, socket)
- Dynamic capability registration and initialization
- CLI tooling to speed up agent development and builds

---

## Getting Started

### Prerequisites

- [Docker](https://www.docker.com/)
- [Rust](https://www.rust-lang.org/)
- Git

### Install

```bash
git clone https://github.com/your-org/keiros.git
cd keiros
cargo install --path .
```

---

## Usage (Quickstart)

### 0) Terms you’ll see
- **Profile**: a YAML file that declares target, release/strip, and which modules/comms are enabled.
- **Teamserver IP / Listener port**: where the agent will register and poll commands (your C2 endpoint).

### 1) Initialize an agent
```bash
./keiros init --agent-name IcyBear
# Creates a new Cargo bin with a minimal agent skeleton
```

### 2) Pick/adjust a build profile
Create `build_profiles/linux_http.yaml`:
```yaml
name: linux_http
target: x86_64-unknown-linux-musl
release: true
strip: true
enabled_features:    # legacy naming; migrating to modules
  - register_agent
  - execute
  - http
```

> Tip: For Windows, set `target: x86_64-pc-windows-gnu` (or `…-msvc` if your toolchain supports it).

### 3) (Legacy) Add a capability
```bash
./keiros feature new --name "pivot"
# Generates a stub and wires it into mod.rs (will become module generator soon)
```

### 4) Build
```bash
./keiros build   --profile linux_http   --teamserver-ip 10.0.0.5   --listener-port 8080
```
**Flags explained**
- `--profile` → which YAML in `build_profiles/` to use
- `--teamserver-ip`, `--listener-port` → baked into the agent’s comms config

### 5) Clean (Docker images/artifacts)
```bash
./keiros clean
# Removes generated Docker images / build artifacts created by Keiros
```

### 6) Run against a stub C2 (smoke test)
- Start your C2 (or a stub that accepts register/poll routes).
- Execute the built agent on a test host and confirm:
  - it **registers** (ID, IP, hostname, status, protocol)
  - it **polls** for tasks
  - it **executes** a trivial task and **reports** the result

> If you don’t have a C2 handy, document a tiny stub (e.g., a simple HTTP handler with `/register` and `/poll`) in `docs/stubs/` so users can verify the beacon loop locally.

---

## Command reference (concise)

| Command | Purpose | Notes |
|---|---|---|
| `./keiros init --agent-name <name>` | Scaffold a new agent | Creates a Cargo bin project. |
| `./keiros feature new --name <cap>` | (Legacy) add a capability | Will be replaced by module generator. |
| `./keiros build --profile <p> --teamserver-ip <ip> --listener-port <port>` | Build an agent | Uses YAML profile; embeds comms config. |
| `./keiros clean` | Clean generated Docker images/artifacts | Frees space; resets build environment. |

---

## Common pitfalls

- **“features vs modules”**: docs say “features”; it’s the legacy path. Migration is to modules. Call this out wherever the user has to choose.
- **Cross-compile toolchains**: for `x86_64-unknown-linux-musl`, ensure the musl target and linker are installed (and available in Docker if you use the container flow).
- **C2 mismatch**: your agent’s comms (HTTP/socket) must match what your C2 actually serves.

---

## Roadmap (High-Level)

- ✅ Legacy feature system (current)
- 🔄 **Module-based architecture** (in progress)
- 🔧 Unified Docker workflows aligned with the new module system
- 🧪 Test harness & example modules
- 📚 Developer guide for custom modules & comms

---

## License

See `LICENSE` for details. Use only with **explicit authorization**.
