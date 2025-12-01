# Cargo Workspace Setup

This document describes the Cargo workspace configuration for the Open CoreUI project.

## Structure

```
open-chatui/
├── Cargo.toml              # Workspace root configuration
├── backend/
│   └── Cargo.toml          # Backend package
└── src-tauri/
    └── Cargo.toml          # Tauri desktop package
```

## Workspace Configuration

### Root Cargo.toml

The workspace is defined in the root `Cargo.toml` with the following structure:

```toml
[workspace]
resolver = "2"
members = [
    "backend",
    "src-tauri"
]

[workspace.package]
version = "0.9.6"
authors = ["OpenWebUI Contributors"]
edition = "2021"
homepage = "https://github.com/sternelee/open-chatui"
repository = "https://github.com/sternelee/open-chatui"
license = "MIT"

[workspace.dependencies]
# Shared dependencies are defined here
# ...
```

### Package Configuration

Both `backend/Cargo.toml` and `src-tauri/Cargo.toml` use workspace dependencies:

```toml
[package]
name = "package-name"
version.workspace = true
authors.workspace = true
edition.workspace = true

[dependencies]
# Use workspace dependencies
some-dependency = { workspace = true }
```

## Benefits

1. **Unified Dependency Management**: Common dependencies are defined once in the workspace root
2. **Consistent Versions**: Ensures both packages use compatible dependency versions
3. **Simplified Builds**: Build commands work at the workspace level
4. **Shared Configuration**: Common package metadata is inherited

## Usage

### Build all packages:
```bash
cargo build --workspace
```

### Build specific package:
```bash
cargo build -p open-webui-rust    # Backend
cargo build -p open-coreui-desktop # Tauri desktop
```

### Check all packages:
```bash
cargo check --workspace
```

### Test all packages:
```bash
cargo test --workspace
```

### Update workspace dependencies:
```bash
cargo update -p <dependency-name>
```

## Workspace Members

1. **backend** (`open-webui-rust`): Actix-web backend server with comprehensive API
2. **src-tauri** (`open-coreui-desktop`): Tauri desktop application with integrated backend

## Shared Dependencies

The workspace defines common dependencies for:
- Web framework (actix-web)
- Async runtime (tokio)
- Database (sqlx)
- Serialization (serde, serde_json)
- Authentication (jsonwebtoken, uuid)
- HTTP client (reqwest)
- Configuration and logging (dotenvy, tracing)
- Date/time handling (chrono)
- Error handling (thiserror, anyhow)
- Utilities (regex, url, mime)
- OpenAI integration (async-openai)
- Tauri dependencies (tauri, tauri plugins)

## Profile Configuration

Release profiles are defined at the workspace level:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

This ensures consistent optimization settings across all workspace members.