# neuxcfg

A tiny, security-conscious library for managing an application's configuration directory.

`neuxcfg` creates a dedicated configuration folder inside the user's system-standard configuration directory and enforces strict file permissions on Unix systems. It is designed for applications that store sensitive data such as API keys, tokens, or database credentials.

## Features

- **Automatic path resolution** — Uses the `dirs` crate to locate the correct configuration directory for the current platform (XDG on Linux, Application Support on macOS, AppData on Windows).
- **Idempotent initialisation** — Calling `init()` multiple times is safe; existing files are never overwritten, and missing components are created only once.
- **Unix permission hardening** — The root directory is set to `0700` (owner-only access) and the default `config.cfg` file is set to `0600` (owner read/write). Existing files are re-permissioned to `0600` to correct any accidental weakening.
- **Custom root paths** — When the default location is not suitable, an arbitrary root directory can be provided via `with_root()`.
- **Clear error handling** — All errors are represented by the `NeuxcfgError` enum, which implements `Display`, `Debug`, `Error`, and `PartialEq`.
- **No unsafe code** — The implementation relies entirely on safe Rust APIs.

## Installation

Add `neuxcfg` to your `Cargo.toml`:

```toml
[dependencies]
neuxcfg = "0.1.0"
```

## Quick Start

```rust
use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the default system config directory.
    let cfg = Neuxcfg::new()?;
    cfg.init()?;

    println!("Configuration root: {:?}", cfg.root());
    Ok(())
}
```

After running this, the directory `~/.config/neuxcfg` (Linux) or its platform equivalent will exist, containing an empty `config.cfg` with owner-only permissions.

## API Overview

### Creating an instance

#### `Neuxcfg::new() -> Result<Self, NeuxcfgError>`

Resolves the default system configuration directory and appends `"neuxcfg"`. Returns `ConfigDirNotFound` if the platform cannot determine a config directory (e.g., missing `$HOME` on Linux).

```rust
let cfg = Neuxcfg::new()?;
```

#### `Neuxcfg::with_root(root: PathBuf) -> Self`

Constructs an instance pointing to an arbitrary directory. No validation is performed until `init()` is called. Useful for testing or non-standard setups.

```rust
let cfg = Neuxcfg::with_root("/opt/myapp/config".into());
```

### Initialisation

#### `cfg.init() -> Result<(), NeuxcfgError>`

Creates the root directory (and all parent directories) if they do not exist. On Unix, sets the root directory permissions to `0700`. Then it attempts to create `config.cfg` inside the root using `OpenOptions::create_new(true)`. If the file already exists, it is left untouched; if it is newly created, or if it already existed, Unix permissions are set to `0600`.

Calling `init()` more than once is safe and effectively a no‑op after the first successful call.

```rust
cfg.init()?; // first call: creates directories and file
cfg.init()?; // second call: does nothing, still safe
```

### Accessing the root path

#### `cfg.root() -> &Path`

Returns a reference to the configuration root path. The path is not guaranteed to exist until after `init()` succeeds.

```rust
let config_file = cfg.root().join("config.cfg");
```

## Error Handling

All fallible operations return `Result<_, NeuxcfgError>`, where `NeuxcfgError` is defined as:

```rust
pub enum NeuxcfgError {
    ConfigDirNotFound,
    Io(String),
}
```

- `ConfigDirNotFound` — The platform’s config directory could not be determined.
- `Io(String)` — An I/O error occurred. The original error message is preserved as a `String` to allow `PartialEq` comparisons.

I/O errors from the standard library are automatically converted via `From<std::io::Error>`.

```rust
match Neuxcfg::new() {
    Ok(cfg) => { /* ... */ },
    Err(NeuxcfgError::ConfigDirNotFound) => eprintln!("No config directory found"),
    Err(NeuxcfgError::Io(msg)) => eprintln!("I/O error: {msg}"),
}
```

## Security Model

This library is built for applications that store sensitive configuration data. On Unix systems (Linux, macOS), it enforces:

- **Directory permissions**: `0700` — only the owner can list, enter, or search the root directory.
- **File permissions**: `0600` — only the owner can read or write the `config.cfg` file.

These permission calls are **compiled out on non-Unix platforms** (e.g., Windows), where file security relies on ACLs and is generally considered adequate without further modification. The `create_new(true)` flag provides atomic, race‑free file creation on all platforms.

Permissions are reapplied to an already‑existing `config.cfg` every time `init()` is called, ensuring that any accidental weakening (e.g., by an external tool) is corrected.

## Platform Behaviour

| Aspect                     | Linux / macOS                                 | Windows                                  |
| -------------------------- | --------------------------------------------- | ---------------------------------------- |
| Config directory           | `dirs::config_dir()` (XDG, macOS conventions) | `dirs::config_dir()` (AppData\Roaming)   |
| Root directory creation    | `create_dir_all`                              | `create_dir_all`                         |
| Root directory permissions | Set to `0o700`                                | No change (no‑op)                        |
| `config.cfg` permissions   | Set to `0o600`                                | No change (no‑op)                        |
| File creation method       | `OpenOptions::create_new(true)` (atomic)      | `OpenOptions::create_new(true)` (atomic) |

The library is tested on Linux, macOS, and Windows in CI (if applicable) to ensure consistent behaviour.

## Testing

Run the crate tests with:

```sh
cargo test
```

Documentation examples can be tested with:

```sh
cargo test --doc
```

The test suite uses `tempfile` to create temporary directories and validates permission bits on Unix. No actual user configuration directories are modified during tests.

## License

This project is licensed under the [MIT License](LICENSE).

## Links

- [Repository](https://github.com/neuxdotdev/neuxcfg)
- [Documentation on docs.rs](https://docs.rs/neuxcfg)
- [Crate on crates.io](https://crates.io/crates/neuxcfg)
