// tests/init.rs
//
// Integration tests for the initialisation logic of the `neuxcfg` library.
// These tests use temporary directories, leaving the real configuration
// untouched.

use neuxcfg::Neuxcfg;
use neuxcfg::types::GlobalConfig;
use std::fs;

// ---------------------------------------------------------------------------
// 1. Fresh initialisation creates the expected directory and a valid
//    TOML configuration file containing the library's metadata.
// ---------------------------------------------------------------------------
#[test]
fn test_init_creates_structure() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());

    cfg.init().unwrap();

    // Root directory must exist and be a directory.
    assert!(root.is_dir());

    // The global configuration file must exist.
    let config_file = root.join("config.cfg");
    assert!(config_file.is_file());

    // It must contain valid TOML that deserialises into a GlobalConfig.
    let content = fs::read_to_string(&config_file).unwrap();
    let global: GlobalConfig =
        toml::from_str(&content).expect("global config should be valid TOML");

    // Check that the well‑known fields are populated with the expected values.
    assert_eq!(global.name, "neuxcfg");
    assert_eq!(global.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(global.edition, "2024");
    assert!(!global.authors.is_empty());
    assert_eq!(global.license, "MIT");
    assert!(global.repository.contains("neuxcfg"));
    // Homepage and documentation may be empty, but that is fine.

    // The root() method must return the exact path that was set.
    assert_eq!(cfg.root(), &*root);
}

// ---------------------------------------------------------------------------
// 2. Calling init() multiple times must be idempotent – no error, no
//    overwrite of existing files.
// ---------------------------------------------------------------------------
#[test]
fn test_init_idempotent() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().join("neuxcfg"));

    cfg.init().unwrap();
    cfg.init().unwrap();
    cfg.init().unwrap(); // three times – must not panic or error.
}

// ---------------------------------------------------------------------------
// 3. with_root() must use the supplied path verbatim, without appending
//    an extra "neuxcfg" segment.
// ---------------------------------------------------------------------------
#[test]
fn test_with_root_exact_path() {
    let tmp = tempfile::TempDir::new().unwrap();
    let custom = tmp.path().join("my_config");
    let cfg = Neuxcfg::with_root(custom.clone());
    cfg.init().unwrap();

    assert!(custom.is_dir());
    assert!(custom.join("config.cfg").exists());
}

// ---------------------------------------------------------------------------
// 4. If the root directory already exists but the config file does not,
//    init() must create the file.
// ---------------------------------------------------------------------------
#[test]
fn test_init_adds_config_when_dir_exists() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    fs::create_dir(&root).unwrap();
    let cfg = Neuxcfg::with_root(root.clone());

    cfg.init().unwrap();
    assert!(root.join("config.cfg").is_file());
}

// ---------------------------------------------------------------------------
// 5. When the config file already exists, init() must never alter its
//    contents.  Permissions are tightened on Unix, but the content is
//    left alone.
// ---------------------------------------------------------------------------
#[test]
fn test_init_does_not_overwrite_existing_config() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("config.cfg");
    fs::write(&config_path, "important data").unwrap();

    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();

    let content = fs::read_to_string(&config_path).unwrap();
    assert_eq!(content, "important data");
}

// ---------------------------------------------------------------------------
// 6. Unix: the root directory must have permissions 0o700 and the config
//    file must have permissions 0o600.
// ---------------------------------------------------------------------------
#[test]
#[cfg(unix)]
fn test_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();

    let dir_perm = fs::metadata(&root).unwrap().permissions().mode();
    assert_eq!(dir_perm & 0o777, 0o700, "root directory must be 0700");

    let file_perm = fs::metadata(root.join("config.cfg"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(file_perm & 0o777, 0o600, "config file must be 0600");
}

// ---------------------------------------------------------------------------
// 7. Unix: if permissions have been relaxed externally, a subsequent
//    init() call must restore the strict permissions.
// ---------------------------------------------------------------------------
#[test]
#[cfg(unix)]
fn test_init_fixes_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();

    // Weaken permissions intentionally.
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(root.join("config.cfg"), fs::Permissions::from_mode(0o644)).unwrap();

    // A second init should repair them.
    cfg.init().unwrap();

    let dir_perm = fs::metadata(&root).unwrap().permissions().mode();
    assert_eq!(
        dir_perm & 0o777,
        0o700,
        "root directory should be 0700 again"
    );

    let file_perm = fs::metadata(root.join("config.cfg"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(file_perm & 0o777, 0o600, "config file should be 0600 again");
}
