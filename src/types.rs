use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::Value;
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
impl GlobalConfig {
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
fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
impl ProjectConfig {
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
