use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
}
impl ProjectConfig {
    pub fn new(name: String, path: String) -> Self {
        Self {
            project: ProjectInfo { name, path },
        }
    }
}
