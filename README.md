# neuxcfg

A production-ready library for secure configuration management.

## Overview

`neuxcfg` is a Rust library that provides a robust foundation for managing per-project configuration files. It ensures data integrity through atomic writes, file locking, and automatic backups, while enforcing strict validation and path traversal protection. Designed for applications that need to persist project-specific settings in a safe, concurrent-friendly manner.

Key features include:

- **Atomic configuration writes**: Prevents corruption using temporary files and UUID-based naming.
- **File locking**: Shared and exclusive locks via `fs2` for safe concurrent access.
- **Backup and restore**: Automatic `.bak` creation before modifications with easy rollback.
- **Path security**: Canonicalization checks to prevent directory traversal attacks.
- **Validation engine**: Custom extra fields are validated for allowed types and key formats.
- **Deep merge updates**: Apply partial configuration changes without replacing the entire file.
- **Unix permission hardening**: Directories (`0700`) and files (`0600`) are set automatically on Unix.

## Requirements

- [Rust](https://www.rust-lang.org/) – edition 2021 or later (the library uses edition 2024 by default, but may compile on earlier editions) [TODO: confirm minimum Rust version]
- Dependencies (managed by Cargo):
    - `dirs` – for locating the system configuration directory
    - `toml` – serialization/deserialization of configuration files
    - `serde` – derive macros for `Serialize` and `Deserialize`
    - `uuid` – generation of unique temporary file names
    - `fs2` – cross-platform file locking
    - `thiserror` – ergonomic error type definitions
    - [TODO: confirm exact version constraints from Cargo.toml]

## Installation

Add `neuxcfg` to your `Cargo.toml`:

```toml
[dependencies]
neuxcfg = "0.3.0"  # [TODO: confirm version number; use exact version or git URL]
```

If the crate is not yet published on crates.io, you can use the Git repository directly:

```toml
[dependencies]
neuxcfg = { git = "https://github.com/neuxdotdev/neuxcfg" }
```

## Quick Start

```rust
use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the global config store (creates ~/.config/neuxcfg/)
    let cfg = Neuxcfg::new()?;
    cfg.init()?;

    // Add a new project
    cfg.add_project("my_app")?;

    // Retrieve configuration
    let mut config = cfg.get_project_config("my_app")?;
    println!("Project name: {}", config.project.name);

    // Update configuration with extra data
    let delta = toml::toml! {
        [project]
        api_key = "secret"
    };
    cfg.update_project_config("my_app", delta)?;

    Ok(())
}
```

## Configuration

`neuxcfg` stores two kinds of TOML configuration:

### Global Configuration

Located at `<config_dir>/neuxcfg/config.cfg` (where `<config_dir>` is determined by `dirs::config_dir()`). The default content is derived from Cargo environment variables, but can be modified directly. Structure:

| Key             | Type   | Default                    | Description             |
| --------------- | ------ | -------------------------- | ----------------------- |
| `name`          | string | `"neuxcfg"`                | Package name            |
| `version`       | string | `"0.3.0"`                  | Crate version           |
| `edition`       | string | `"2024"`                   | Rust edition            |
| `description`   | string | `"A production‑ready..."`  | Package description     |
| `authors`       | array  | `["neuxdotdev <...>"]`     | List of authors         |
| `license`       | string | `"MIT"`                    | SPDX license identifier |
| `repository`    | string | `"https://github.com/..."` | Source repository URL   |
| `homepage`      | string | `""`                       | Project homepage        |
| `documentation` | string | `""`                       | Documentation URL       |

Source: [`src/types.rs`](./src/types.rs) – `GlobalConfig` struct and `GlobalConfig::from_cargo()`.

### Project Configuration

Stored at `<config_dir>/neuxcfg/<project_name>/<project_name>.config.cfg`. Example:

```toml
[project]
name = "my_app"
path = "/path/to/project"
# Extra fields allowed (flattened into project table)
api_key = "secret"
timeout = 30
enabled = true
```

- `name` and `path` are mandatory.
- Extra keys must not start with `_` or contain `.`.
- Supported value types: string, integer, float, boolean, tables, arrays of those types.

Source: [`src/types.rs`](./src/types.rs) – `ProjectConfig` and `ProjectInfo`; [`src/validate.rs`](./src/validate.rs) – validation rules.

## Usage

### Initialization and Custom Root

```rust
use neuxcfg::Neuxcfg;

// Use default system config directory
let cfg = Neuxcfg::new()?;

// Or specify a custom root path
let cfg = Neuxcfg::with_root("/custom/path".into());
cfg.init()?; // Creates the root if not present
```

### Managing Projects

```rust
// Add a project
cfg.add_project("web_server")?;

// Check existence
assert!(cfg.project_exists("web_server")?);

// List all projects
let projects = cfg.list_projects()?;
// ["web_server"]

// Remove a project (deletes directory and config)
cfg.remove_project("web_server")?;
```

### Reading and Writing Configurations

```rust
// Read full configuration
let config = cfg.get_project_config("my_app")?;
println!("Path: {}", config.project.path);

// Write a completely new configuration (triggers backup)
let new_config = ProjectConfig::new("my_app".into(), "/new/path".into());
cfg.set_project_config("my_app", &new_config)?;

// Apply partial update (deep merge)
let delta = toml::toml! {
    [project]
    debug_mode = true
    limits.max_connections = 100
};
cfg.update_project_config("my_app", delta)?;
```

### Backup and Restore

```rust
// Backup current config file → creates <project>.config.cfg.bak
let backup_path = cfg.backup_project("my_app")?;

// ... make changes ...

// Restore from backup
cfg.restore_project("my_app")?;
```

All write operations (`set_project_config`, `update_project_config`, `init`, `add_project`) are atomic: they write to a temporary file, flush, and then rename. File locking is applied during reads and writes to prevent races.

## API Reference

### `Neuxcfg` struct

- `Neuxcfg::new() -> Result<Self, NeuxcfgError>` – Creates a new instance using the system config directory.
- `Neuxcfg::with_root(root: PathBuf) -> Self` – Creates an instance with a custom root directory.
- `fn init(&self) -> Result<(), NeuxcfgError>` – Creates the root directory (if missing) and writes default global config.
- `fn root(&self) -> &Path` – Returns the root path.
- `fn add_project(&self, name: &str) -> Result<(), NeuxcfgError>` – Creates a new project directory and default config.
- `fn remove_project(&self, name: &str) -> Result<(), NeuxcfgError>` – Removes the project directory and all its contents.
- `fn list_projects(&self) -> Result<Vec<String>, NeuxcfgError>` – Lists all valid project names.
- `fn project_exists(&self, name: &str) -> Result<bool, NeuxcfgError>` – Checks if a project directory exists.
- `fn get_project_config(&self, name: &str) -> Result<ProjectConfig, NeuxcfgError>` – Reads and deserializes the project config.
- `fn set_project_config(&self, name: &str, config: &ProjectConfig) -> Result<(), NeuxcfgError>` – Overwrites the project config (atomic write + backup).
- `fn update_project_config(&self, name: &str, delta: toml::Value) -> Result<(), NeuxcfgError>` – Merges a TOML value into the existing config.
- `fn backup_project(&self, name: &str) -> Result<PathBuf, NeuxcfgError>` – Creates a backup of the config file.
- `fn restore_project(&self, name: &str) -> Result<(), NeuxcfgError>` – Restores the config from backup.

### `NeuxcfgError` enum

Variants (implement `std::error::Error` and `PartialEq`):

| Variant                | Description                                  |
| ---------------------- | -------------------------------------------- |
| `ConfigDirNotFound`    | System config directory cannot be determined |
| `Io(String)`           | Wraps an I/O error                           |
| `InvalidProjectName`   | Project name contains illegal characters     |
| `ProjectAlreadyExists` | Project directory already present            |
| `ProjectNotFound`      | Project does not exist                       |
| `TomlParse`            | Failed to parse a TOML string                |
| `TomlSerialize`        | Failed to serialize a value to TOML          |
| `PathTraversal`        | Detected attempt to access outside root      |
| `ValidationError`      | Invalid extra field in configuration         |
| `LockError`            | File locking failure                         |

### Public data types

- **`GlobalConfig`** – Represents the global configuration file (see [Configuration](#global-configuration)).
- **`ProjectConfig`** – Contains a `project` field of type `ProjectInfo`.
- **`ProjectInfo`** – Holds `name`, `path`, and a `HashMap<String, toml::Value>` for extra fields.

All types derive `Debug`, `PartialEq`, `Serialize`, `Deserialize`.

### Helper functions (private modules, exposed internally)

The following functions are **not** part of the public API but are used by `Neuxcfg` methods:

- `atomic::atomic_write` – Atomic file write with temp file and rename.
- `backup::backup_project_config` / `restore_project_config` – Backup/restore logic.
- `lock::FileLock` – RAII-based file lock (shared/exclusive).
- `merge::deep_merge` – Recursive merge of `toml::Value` tables.
- `validate::validate_extra` – Ensures extra keys and values are allowed.

If you need these utilities standalone, consider opening a feature request or exposing them via feature flags.

## Testing

Run the standard Rust test suite:

```bash
cargo test
```

[TODO: requires confirmation – no test files were found in the provided source. The command may fail if no tests are present. Add unit and integration tests as needed.]

## Project Structure

```
.
├── src
│   ├── lib.rs          # Main library: Neuxcfg struct and public API
│   ├── error.rs        # Error types (NeuxcfgError)
│   ├── types.rs        # Configuration data structures
│   ├── atomic.rs       # Atomic file write utility
│   ├── backup.rs       # Backup and restore helpers
│   ├── lock.rs         # File locking (FileLock)
│   ├── merge.rs        # Deep merge for TOML values
│   └── validate.rs     # Extra field validator
├── Cargo.toml          # (not provided, assumed)
└── README.md
```

## Contributing

Contributions are welcome. Please follow standard Rust open-source practices:

1. Fork the repository.
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Commit with conventional format: `feat: add new capability`
4. Ensure `cargo test` and `cargo fmt` pass.
5. Push and open a Pull Request.

For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License – see the [LICENSE](./LICENSE) file for details (based on default `GlobalConfig` value). [TODO: verify that a LICENSE file exists in the repository.]

## References

- [TOML specification](https://toml.io)
- [serde](https://serde.rs)
- [dirs crate](https://crates.io/crates/dirs)
- [fs2 crate](https://crates.io/crates/fs2)
- [uuid crate](https://crates.io/crates/uuid)
- [thiserror crate](https://crates.io/crates/thiserror)

---

> **Repository**: [https://github.com/neuxdotdev/neuxcfg](https://github.com/neuxdotdev/neuxcfg)  
> **Issues**: [https://github.com/neuxdotdev/neuxcfg/issues](https://github.com/neuxdotdev/neuxcfg/issues)
