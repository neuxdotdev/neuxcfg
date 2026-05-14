# neuxcfg

[![Crates.io](https://img.shields.io/crates/v/neuxcfg?style=for-the-badge&logo=rust&label=version)](https://crates.io/crates/neuxcfg)
[![Documentation](https://img.shields.io/docsrs/neuxcfg?style=for-the-badge&logo=docsdotrs&label=docs.rs)](https://docs.rs/neuxcfg)
[![License](https://img.shields.io/crates/l/neuxcfg?style=for-the-badge)](LICENSE)
[![Minimum rustc](https://img.shields.io/badge/rustc-1.60%2B-informational?style=for-the-badge&logo=rust)](https://rust-lang.org)
[![CI Status](https://img.shields.io/github/actions/workflow/status/neuxdotdev/neuxcfg/ci.yml?style=for-the-badge&logo=githubactions)](https://github.com/neuxdotdev/neuxcfg/actions)
[![Code Coverage](https://img.shields.io/codecov/c/github/neuxdotdev/neuxcfg?style=for-the-badge&logo=codecov&token=YOUR_CODECOV_TOKEN)](https://codecov.io/gh/neuxdotdev/neuxcfg)
[![LoC](https://tokei.rs/b1/github/neuxdotdev/neuxcfg?category=code&style=flat)](https://github.com/neuxdotdev/neuxcfg)

> A tiny, security-first library for managing your application’s configuration directory and **project-specific configurations** — all stored safely under the user’s standard config path.

`neuxcfg` creates a dedicated `neuxcfg/` folder inside the system‑standard configuration directory (e.g., `~/.config/neuxcfg` on Linux) and ensures **strict Unix permissions** for every directory and file. It goes beyond a single global config by giving you a **project registry** — each project gets its own subdirectory with a TOML file that supports custom fields, making it a perfect companion for CLI tools, daemons, or any Rust application that needs to persist settings, tokens, or runtime data.

---

## Features at a Glance

- **Cross‑platform config root** – Uses `dirs::config_dir()` (XDG on Linux, Application Support on macOS, AppData on Windows).
- **Idempotent initialisation** – Call `init()` as many times as you want; existing files are **never overwritten**.
- **Unix permission hardening** – Root dir `0700`, global config `0600`, project dirs `0700`, project configs `0600`. Permissions are **re‑applied** on every `init()`.
- **Project management** – `add_project`, `remove_project`, `list_projects`, `get_project_config`, `set_project_config` — a full CRUD system on top of the filesystem.
- **Custom fields** – Each project config can carry arbitrary TOML data (`extra` map) alongside the mandatory `name` and `path`.
- **Global metadata** – The root `config.cfg` automatically captures the library’s own name, version, edition, authors, and repository links.
- **Zero unsafe code** – 100% safe Rust.
- **Rich error type** – `NeuxcfgError` derives `Display`, `Debug`, `Error`, and `PartialEq`. Converts from `std::io::Error` and TOML errors automatically.
- **Thoroughly tested** – Integration tests for initialisation (11 tests) and project lifecycle (13 tests), covering permissions, idempotency, path traversal, malformed TOML, and edge cases.

---

## Installation

Add to your `Cargo.toml`:

```bash
cargo add neuxcfg
```

Minimum Supported Rust Version: **1.60.0**

---

## Quick Start

This snippet initialises the configuration root, creates a project called `"myapp"`, and writes a custom key.

```rust
use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Neuxcfg::new()?;
    cfg.init()?;

    // Add a new project
    cfg.add_project("myapp")?;

    // Read its default config
    let mut config = cfg.get_project_config("myapp")?;
    println!("Default path: {}", config.project.path);

    // Add a custom field
    config.project.extra.insert(
        "port".into(),
        toml::Value::Integer(8080),
    );
    cfg.set_project_config("myapp", &config)?;

    // List all projects
    println!("All projects: {:?}", cfg.list_projects()?);
    Ok(())
}
```

After running, your filesystem will look like this:

```
~/.config/neuxcfg/
├── config.cfg                     # global metadata (name, version, authors, etc.)
└── myapp/
    └── myapp.config.cfg           # [project] name, path, and any custom fields
```

---

## Full API Reference

### `Neuxcfg` struct

The central manager. Obtain an instance and then call `init()` once. All project methods require the root to be initialised (though they will attempt to create missing parent directories if needed).

#### Constructors

| Method                                         | Description                                                                                                               |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `Neuxcfg::new() -> Result<Self, NeuxcfgError>` | Points to the system’s config directory + `"neuxcfg"`. Returns `ConfigDirNotFound` if the directory cannot be determined. |
| `Neuxcfg::with_root(root: PathBuf) -> Self`    | Uses an arbitrary custom path. No validation is performed until `init()` is called. Ideal for testing.                    |

#### Core Methods

| Method                                   | Description                                                                                                                                                                                                                         |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cfg.init() -> Result<(), NeuxcfgError>` | Creates the root directory (`0700` on Unix) and the global `config.cfg` with full metadata (if it doesn’t exist). Already‑existing files are left untouched, but permissions are tightened to `0600` (file) and `0700` (directory). |
| `cfg.root() -> &Path`                    | Returns a reference to the root directory path.                                                                                                                                                                                     |

#### Project Management

| Method                                                                                   | Description                                                                                                                                                                                               |
| ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cfg.add_project(name: &str) -> Result<(), NeuxcfgError>`                                | Creates a subdirectory `<name>` inside the root with permission `0700`, and writes a default `ProjectConfig` (name + path) into `<name>.config.cfg`. Rejects invalid names (empty, `..`, `/`, `\`, null). |
| `cfg.remove_project(name: &str) -> Result<(), NeuxcfgError>`                             | Deletes the entire project directory after validating the path is still inside the root (symlink traversal protection).                                                                                   |
| `cfg.list_projects() -> Result<Vec<String>, NeuxcfgError>`                               | Scans the root directory and returns all valid project names.                                                                                                                                             |
| `cfg.project_exists(name: &str) -> Result<bool, NeuxcfgError>`                           | Checks whether a project subdirectory exists.                                                                                                                                                             |
| `cfg.get_project_config(name: &str) -> Result<ProjectConfig, NeuxcfgError>`              | Reads the project’s `{name}.config.cfg` and parses it as TOML.                                                                                                                                            |
| `cfg.set_project_config(name: &str, config: &ProjectConfig) -> Result<(), NeuxcfgError>` | Overwrites the project’s config file with the given structure. On Unix, permissions are reset to `0600` after writing.                                                                                    |

---

## Data Types

All types live in the `neuxcfg::types` module.

### `GlobalConfig`

Automatically written to the root `config.cfg` when `init()` creates it for the first time.

```rust
pub struct GlobalConfig {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: String,
    pub homepage: String,
    pub documentation: String,
}
```

You can also construct it manually (e.g., for testing) via `GlobalConfig::from_cargo()` which reads compile‑time environment variables.

### `ProjectConfig`

```rust
pub struct ProjectConfig {
    pub project: ProjectInfo,
}

pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}
```

The `extra` map uses `serde(flatten)`, so any keys you add become top‑level TOML entries under `[project]`. For instance, inserting `("port", Integer(8080))` will produce:

```toml
[project]
name = "myapp"
path = "/home/user/.config/neuxcfg/myapp"
port = 8080
```

This gives you complete flexibility to store arbitrary structured data per project.

---

## Security Model

`neuxcfg` is designed for applications that handle sensitive data (API keys, tokens, secrets). On Unix systems (Linux, macOS, Android):

- **Root directory** → `0700` – only the owner can enter or list its contents.
- **Global config file** → `0600` – only the owner can read/write.
- **Project directories** → `0700` – same restriction.
- **Project config files** → `0600`.

Every `init()` call re‑applies these permissions, so accidental weakening (e.g., by an external tool) is corrected. Windows permission calls are compiled out; the existing ACL model is considered sufficient.

**Path traversal protection:** all project name inputs are validated, and symlinks that point outside the root directory are detected and rejected with a `PathTraversal` error.

---

## Cross‑Platform Behaviour

| Platform    | Config Root Example                        | Permissions     |
| ----------- | ------------------------------------------ | --------------- |
| **Linux**   | `~/.config/neuxcfg/`                       | `0700` / `0600` |
| **macOS**   | `~/Library/Application Support/neuxcfg/`   | `0700` / `0600` |
| **Windows** | `C:\Users\<User>\AppData\Roaming\neuxcfg\` | (no change)     |

All directory and file creation is performed through `OpenOptions::create_new(true)`, ensuring atomic, race‑free file creation on every platform.

---

## Version History & Roadmap

### Current (v0.2.0)

- Global metadata in `config.cfg`.
- Full project CRUD: `add_project`, `remove_project`, `list_projects`, `get_project_config`, `set_project_config`.
- Custom extra fields per project.
- Path traversal protection (symlinks).
- Comprehensive test suite (24 integration tests).

### Upcoming (v0.3.0)

- Per‑key access to project TOML (get/set/delete individual keys).
- Advisory file locking (flock) for concurrent access.
- Atomic write (write‑to‑temp + rename) for even safer config updates.

### Future (v1.0.0)

- Stable API commitment.
- Windows‑native permission hardening (ACL wrappers) – opt‑in.
- Environment variable fallback (`get_value_or_env`).

---

## Running the Tests

```bash
cargo test
```

All tests use temporary directories (`tempfile`) — your real configuration is never touched. The suite covers:

- Fresh initialisation & idempotency.
- Permission setting and repair (Unix).
- Project lifecycle: add, list, get, set, remove.
- Invalid project names, missing projects, duplicate projects.
- Path traversal via symlinks.
- Malformed TOML handling.
- Custom field round‑trips and overwrite semantics.

---

## Contributing

Contributions are welcome! Please open an issue or pull request on [GitHub](https://github.com/neuxdotdev/neuxcfg). Ensure tests pass with `cargo test --all-targets --all-features` and that `cargo clippy -- -D warnings` is clean.

---

## License

Licensed under the [MIT License](LICENSE). See the `LICENSE` file for full text.

---

## Links

- [Repository](https://github.com/neuxdotdev/neuxcfg)
- [Documentation on docs.rs](https://docs.rs/neuxcfg)
- [Crate on crates.io](https://crates.io/crates/neuxcfg)
