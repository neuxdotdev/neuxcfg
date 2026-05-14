use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::Value;

/// Global configuration stored in the root `config.cfg`.
///
/// Contains metadata about the neuxcfg library itself (or the embedding
/// application), typically populated from Cargo environment variables at
/// initialisation time.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Package name (e.g., "neuxcfg").
    pub name: String,
    /// Crate version.
    pub version: String,
    /// Rust edition used (e.g., "2024").
    pub edition: String,
    /// Short description of the crate.
    pub description: String,
    /// List of authors (separated by `:` in the environment variable, split here).
    pub authors: Vec<String>,
    /// SPDX license identifier (e.g., "MIT").
    pub license: String,
    /// URL of the repository.
    pub repository: String,
    /// URL of the project homepage.
    pub homepage: String,
    /// URL of the online documentation.
    pub documentation: String,
}

impl GlobalConfig {
    /// Creates a `GlobalConfig` by reading standard Cargo environment variables.
    ///
    /// Each field falls back to a sensible default if the corresponding
    /// environment variable is not set. This is intended to be called during
    /// [`Neuxcfg::init`] to seed the global configuration file.
    ///
    /// # Default values (if env var missing)
    /// - `name`: "neuxcfg"
    /// - `version`: "0.2.0"
    /// - `edition`: "2024"
    /// - `description`: "library for managing an application’s configuration directory."
    /// - `authors`: "neuxdotdev <neuxdev1@gmail.com>" (split by ':')
    /// - `license`: "MIT"
    /// - `repository`: "https://github.com/neuxdotdev/neuxcfg"
    /// - `homepage`: ""
    /// - `documentation`: ""
    pub fn from_cargo() -> Self {
        Self {
            name: env_or_default("CARGO_PKG_NAME", "neuxcfg"),
            version: env_or_default("CARGO_PKG_VERSION", "0.2.0"),
            edition: env_or_default("CARGO_EDITION", "2024"),
            description: env_or_default(
                "CARGO_PKG_DESCRIPTION",
                "library for managing an application’s configuration directory.",
            ),
            authors: env_or_default("CARGO_PKG_AUTHORS", "neuxdotdev <neuxdev1@gmail.com>")
                .split(':')
                .map(|s| s.to_string())
                .collect(),
            license: env_or_default("CARGO_PKG_LICENSE", "MIT"),
            repository: env_or_default(
                "CARGO_PKG_REPOSITORY",
                "https://github.com/neuxdotdev/neuxcfg",
            ),
            homepage: env_or_default("CARGO_PKG_HOMEPAGE", ""),
            documentation: env_or_default("CARGO_PKG_DOCUMENTATION", ""),
        }
    }
}

/// Helper to retrieve an environment variable or a default value.
///
/// # Arguments
/// * `key` - The name of the environment variable.
/// * `default` - The fallback string if the variable is not set.
fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Per‑project configuration stored in `<project>.config.cfg`.
///
/// Wraps a [`ProjectInfo`] struct. Future versions may include additional
/// top‑level fields.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Core information about the project.
    pub project: ProjectInfo,
}

/// Detailed information about a single project.
///
/// This is serialised directly inside the `ProjectConfig`. It contains the
/// project’s logical name, the absolute path to its working directory, and
/// an open‑ended set of extra key–value pairs that are flattened into the
/// TOML table.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// The project’s unique name (matches the directory name).
    pub name: String,
    /// The absolute path to the project’s directory on disk.
    pub path: String,
    /// Any additional configuration fields. Because this uses
    /// `#[serde(flatten)]`, extra keys appear at the same level as `name`
    /// and `path` in the serialised TOML.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ProjectConfig {
    /// Creates a new `ProjectConfig` with the given name and path.
    ///
    /// The `extra` map is initialised empty.
    pub fn new(name: String, path: String) -> Self {
        Self {
            project: ProjectInfo {
                name,
                path,
                extra: HashMap::new(),
            },
        }
    }
}
