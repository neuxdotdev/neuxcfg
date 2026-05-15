use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::Value;

/// Global configuration for the `neuxcfg` installation itself.
///
/// This is typically written once during [`Neuxcfg::init`](crate::Neuxcfg::init)
/// and contains metadata about the library (name, version, authors, etc.).
/// Values are populated from Cargo environment variables when available.
///
/// # Examples
///
/// ```rust
/// use neuxcfg::GlobalConfig;
///
/// let cfg = GlobalConfig::from_cargo();
/// assert!(!cfg.name.is_empty());
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Package name (e.g., `"neuxcfg"`).
    pub name: String,
    /// Crate version (e.g., `"0.3.0"`).
    pub version: String,
    /// Rust edition (e.g., `"2024"`).
    pub edition: String,
    /// Short description.
    pub description: String,
    /// List of authors, semicolon-separated in the default.
    pub authors: Vec<String>,
    /// SPDX license identifier.
    pub license: String,
    /// URL of the source repository.
    pub repository: String,
    /// Project homepage URL.
    pub homepage: String,
    /// Documentation URL.
    pub documentation: String,
}

impl GlobalConfig {
    /// Constructs a `GlobalConfig` from Cargo environment variables.
    ///
    /// Reads the following variables (with fallbacks):
    ///
    /// | Variable                 | Default                       |
    /// |--------------------------|-------------------------------|
    /// | `CARGO_PKG_NAME`         | `"neuxcfg"`                   |
    /// | `CARGO_PKG_VERSION`      | `"0.3.0"`                     |
    /// | `CARGO_EDITION`          | `"2024"`                      |
    /// | `CARGO_PKG_DESCRIPTION`  | `"A production-ready ..."`    |
    /// | `CARGO_PKG_AUTHORS`      | `"neuxdotdev <neuxdev1...>"`  |
    /// | `CARGO_PKG_LICENSE`      | `"MIT"`                       |
    /// | `CARGO_PKG_REPOSITORY`   | `"https://github.com/..."`    |
    /// | `CARGO_PKG_HOMEPAGE`     | `""`                          |
    /// | `CARGO_PKG_DOCUMENTATION`| `""`                          |
    ///
    /// Authors are split on `:` to form a list.
    pub fn from_cargo() -> Self {
        Self {
            name: env_or_default("CARGO_PKG_NAME", "neuxcfg"),
            version: env_or_default("CARGO_PKG_VERSION", "0.3.0"),
            edition: env_or_default("CARGO_EDITION", "2024"),
            description: env_or_default(
                "CARGO_PKG_DESCRIPTION",
                "A production‑ready library for secure configuration management.",
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

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Configuration for a specific project.
///
/// Wraps a single [`ProjectInfo`] under the `project` key.
/// Use [`ProjectConfig::new`] for a minimal valid configuration.
///
/// # Serialization
///
/// Serialises as:
///
/// ```toml
/// [project]
/// name = "..."
/// path = "..."
/// # extra fields appear inline
/// key = "value"
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Metadata and extra fields for the project.
    pub project: ProjectInfo,
}

/// Core project metadata and custom extra fields.
///
/// `name` and `path` are mandatory. Additional fields are captured in the
/// `extra` map, which is flattened into the `[project]` table during
/// serialisation. Keys starting with `_` or containing `.` are rejected by
/// the validator.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name (must be a valid directory name).
    pub name: String,
    /// Filesystem path associated with the project.
    pub path: String,
    /// Arbitrary extra fields (string, integer, float, boolean, table, array).
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ProjectConfig {
    /// Creates a new `ProjectConfig` with the given name and path, and no extra fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use neuxcfg::ProjectConfig;
    ///
    /// let cfg = ProjectConfig::new("myapp".into(), "/opt/myapp".into());
    /// assert_eq!(cfg.project.name, "myapp");
    /// assert!(cfg.project.extra.is_empty());
    /// ```
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
