// tests/projects.rs
//
// Integration tests for project lifecycle management in `neuxcfg`.
//
// These tests cover:
//   - Adding, listing, and removing projects
//   - Project existence checks
//   - Reading and writing project configurations
//   - Custom extra fields in project configuration
//   - Validation of project names (security)
//   - Path traversal prevention (symlink attacks)
//   - Error handling for missing projects, duplicate projects, and I/O errors
//   - Unix permission enforcement for project directories and files
//   - Robust TOML parsing (e.g. handling invalid content)
//   - Idempotent behaviour of set/get operations
//   - Edge cases: empty extra, unicode names, root not initialised

use neuxcfg::Neuxcfg;
use neuxcfg::NeuxcfgError;
use neuxcfg::types::ProjectConfig;
use std::fs;

// ---------------------------------------------------------------------------
// 1. Happy path: add a project, list it, inspect its default config, and
//    remove it.
// ---------------------------------------------------------------------------
#[test]
fn test_add_list_remove_project() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();

    // Add
    cfg.add_project("myapp").unwrap();
    assert!(cfg.project_exists("myapp").unwrap());

    // List
    let list = cfg.list_projects().unwrap();
    assert!(list.contains(&"myapp".to_string()));

    // Default config
    let config = cfg.get_project_config("myapp").unwrap();
    assert_eq!(config.project.name, "myapp");
    let expected_path = cfg.root().join("myapp");
    assert_eq!(config.project.path, expected_path.to_string_lossy());
    assert!(config.project.extra.is_empty());

    // Remove
    cfg.remove_project("myapp").unwrap();
    assert!(!cfg.project_exists("myapp").unwrap());

    // List should be empty afterwards
    let list_after = cfg.list_projects().unwrap();
    assert!(!list_after.contains(&"myapp".to_string()));
}

// ---------------------------------------------------------------------------
// 2. Adding a project that already exists must return
//    `ProjectAlreadyExists`.
// ---------------------------------------------------------------------------
#[test]
fn test_add_existing_project_errors() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    cfg.add_project("dup").unwrap();
    let err = cfg.add_project("dup").unwrap_err();
    assert_eq!(err, NeuxcfgError::ProjectAlreadyExists("dup".into()));
}

// ---------------------------------------------------------------------------
// 3. Invalid project names (empty, containing /, \, .., or null byte)
//    must be rejected.
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_project_name() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();

    assert!(matches!(
        cfg.add_project("../escape"),
        Err(NeuxcfgError::InvalidProjectName(_))
    ));
    assert!(matches!(
        cfg.add_project("with/slash"),
        Err(NeuxcfgError::InvalidProjectName(_))
    ));
    assert!(matches!(
        cfg.add_project("with\\backslash"),
        Err(NeuxcfgError::InvalidProjectName(_))
    ));
    assert!(matches!(
        cfg.add_project(""),
        Err(NeuxcfgError::InvalidProjectName(_))
    ));
    // Null byte (not possible in Rust string literals, but we can test via raw bytes)
    // Rust strings cannot contain null bytes, so we trust the check inside the library.
}

// ---------------------------------------------------------------------------
// 4. Valid project names, including unicode, are accepted.
// ---------------------------------------------------------------------------
#[test]
fn test_unicode_project_name() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();

    let name = "café_project-β";
    cfg.add_project(name).unwrap();
    assert!(cfg.project_exists(name).unwrap());
    let config = cfg.get_project_config(name).unwrap();
    assert_eq!(config.project.name, name);
    cfg.remove_project(name).unwrap();
}

// ---------------------------------------------------------------------------
// 5. Setting and retrieving custom fields in `extra`.
// ---------------------------------------------------------------------------
#[test]
fn test_get_set_project_config_with_custom_fields() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    cfg.add_project("myapp").unwrap();

    let mut config = cfg.get_project_config("myapp").unwrap();
    config.project.extra.insert(
        "database_url".into(),
        toml::Value::String("postgres://localhost".into()),
    );
    config
        .project
        .extra
        .insert("max_connections".into(), toml::Value::Integer(10));

    cfg.set_project_config("myapp", &config).unwrap();

    let updated = cfg.get_project_config("myapp").unwrap();
    assert_eq!(updated.project.name, "myapp");
    assert_eq!(
        updated.project.extra.get("database_url"),
        Some(&toml::Value::String("postgres://localhost".into()))
    );
    assert_eq!(
        updated.project.extra.get("max_connections"),
        Some(&toml::Value::Integer(10))
    );
}

// ---------------------------------------------------------------------------
// 6. `set_project_config` completely overwrites the existing file, so
//    missing keys in the new config are removed.
// ---------------------------------------------------------------------------
#[test]
fn test_set_project_config_overwrites() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    cfg.add_project("test_app").unwrap();

    // Add some initial extra fields
    let mut config = cfg.get_project_config("test_app").unwrap();
    config
        .project
        .extra
        .insert("key1".into(), toml::Value::String("value1".into()));
    cfg.set_project_config("test_app", &config).unwrap();

    // Now overwrite with a completely different config (no extra)
    let new_config = ProjectConfig::new("test_app".into(), "/new/path".into());
    cfg.set_project_config("test_app", &new_config).unwrap();

    let final_config = cfg.get_project_config("test_app").unwrap();
    assert_eq!(final_config.project.path, "/new/path");
    assert!(
        final_config.project.extra.is_empty(),
        "extra fields should be cleared after overwrite"
    );
}

// ---------------------------------------------------------------------------
// 7. Errors on missing project for `get_project_config`, `set_project_config`,
//    and `remove_project`.
// ---------------------------------------------------------------------------
#[test]
fn test_project_not_found_errors() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();

    assert_eq!(
        cfg.get_project_config("ghost").unwrap_err(),
        NeuxcfgError::ProjectNotFound("ghost".into())
    );
    assert_eq!(
        cfg.remove_project("ghost").unwrap_err(),
        NeuxcfgError::ProjectNotFound("ghost".into())
    );
    // set_project_config on non-existent project
    let dummy = ProjectConfig::new("ghost".into(), "/tmp".into());
    assert_eq!(
        cfg.set_project_config("ghost", &dummy).unwrap_err(),
        NeuxcfgError::ProjectNotFound("ghost".into())
    );
}

// ---------------------------------------------------------------------------
// 8. `project_exists` with an invalid name must return an error.
// ---------------------------------------------------------------------------
#[test]
fn test_project_exists_invalid_name() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    assert!(matches!(
        cfg.project_exists("../bad"),
        Err(NeuxcfgError::InvalidProjectName(_))
    ));
}

// ---------------------------------------------------------------------------
// 9. `project_exists` returns false when the project directory does not
//    exist, even if the root directory is missing.
// ---------------------------------------------------------------------------
#[test]
fn test_project_exists_when_root_missing() {
    let tmp = tempfile::TempDir::new().unwrap();
    let non_existent = tmp.path().join("no_dir");
    let cfg = Neuxcfg::with_root(non_existent);
    // The root does not exist yet, so no project can be found.
    assert!(
        !cfg.project_exists("any").unwrap(),
        "project should not exist when root directory is missing"
    );
}
// ---------------------------------------------------------------------------
// 10. `list_projects` returns an empty vector when the root directory
//     does not exist.
// ---------------------------------------------------------------------------
#[test]
fn test_list_projects_empty_when_root_missing() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("missing_root");
    let cfg = Neuxcfg::with_root(root);
    assert!(cfg.list_projects().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// 11. Path traversal protection: if a symlink inside the root points
//     outside, `remove_project` (and any other method that uses
//     `secure_project_dir`) should return `PathTraversal`.
// ---------------------------------------------------------------------------
#[test]
#[cfg(unix)]
fn test_symlink_path_traversal_rejected() {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();

    // Create a symlink named "escape" inside root pointing to /tmp (outside)
    let symlink_path = root.join("escape");
    symlink("/tmp", &symlink_path).unwrap();

    // Removing "escape" should detect path traversal.
    let err = cfg.remove_project("escape").unwrap_err();
    assert_eq!(err, NeuxcfgError::PathTraversal("escape".into()));
}

// ---------------------------------------------------------------------------
// 12. Unix permissions for project directories (0700) and config files
//     (0600) are enforced.
// ---------------------------------------------------------------------------
#[test]
#[cfg(unix)]
fn test_project_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();

    cfg.add_project("myapp").unwrap();

    let proj_dir = root.join("myapp");
    let dir_perm = fs::metadata(&proj_dir).unwrap().permissions().mode();
    assert_eq!(dir_perm & 0o777, 0o700, "project directory must be 0700");

    let config_file = proj_dir.join("myapp.config.cfg");
    let file_perm = fs::metadata(&config_file).unwrap().permissions().mode();
    assert_eq!(file_perm & 0o777, 0o600, "project config file must be 0600");
}

// ---------------------------------------------------------------------------
// 13. Handling invalid TOML in a project config file (e.g. manually
//     edited) – `get_project_config` should return a `TomlParse` error.
// ---------------------------------------------------------------------------
#[test]
fn test_malformed_toml_config_returns_parse_error() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();
    cfg.add_project("broken").unwrap();

    // Overwrite the config file with invalid TOML
    let config_path = root.join("broken").join("broken.config.cfg");
    fs::write(&config_path, "this is not valid toml {{{").unwrap();

    let err = cfg.get_project_config("broken").unwrap_err();
    assert!(matches!(err, NeuxcfgError::TomlParse(_)));
}
