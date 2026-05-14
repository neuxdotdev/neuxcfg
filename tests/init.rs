use neuxcfg::Neuxcfg;
use std::fs;
#[test]
fn test_init_creates_structure() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();
    assert!(root.is_dir());
    let config_file = root.join("config.cfg");
    assert!(config_file.is_file());
    let content = fs::read_to_string(&config_file).unwrap();
    assert!(content.is_empty());
    assert_eq!(cfg.root(), &*root);
}
#[test]
fn test_init_idempotent() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().join("neuxcfg"));
    cfg.init().unwrap();
    cfg.init().unwrap();
    cfg.init().unwrap(); 
}
#[test]
fn test_with_root_exact_path() {
    let tmp = tempfile::TempDir::new().unwrap();
    let custom = tmp.path().join("my_config");
    let cfg = Neuxcfg::with_root(custom.clone());
    cfg.init().unwrap();
    assert!(custom.is_dir());
    assert!(custom.join("config.cfg").exists());
}
#[test]
fn test_init_adds_config_when_dir_exists() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    fs::create_dir(&root).unwrap();
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();
    assert!(root.join("config.cfg").is_file());
}
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
#[test]
#[cfg(unix)]
fn test_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();
    let dir_perm = fs::metadata(&root).unwrap().permissions().mode();
    assert_eq!(dir_perm & 0o777, 0o700, "Folder root harus 0700");
    let file_perm = fs::metadata(root.join("config.cfg")).unwrap().permissions().mode();
    assert_eq!(file_perm & 0o777, 0o600, "File config harus 0600");
}
#[test]
#[cfg(unix)]
fn test_init_fixes_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("neuxcfg");
    let cfg = Neuxcfg::with_root(root.clone());
    cfg.init().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(root.join("config.cfg"), fs::Permissions::from_mode(0o644)).unwrap();
    cfg.init().unwrap();
    let dir_perm = fs::metadata(&root).unwrap().permissions().mode();
    assert_eq!(dir_perm & 0o777, 0o700, "Folder root harus kembali 0700");
    let file_perm = fs::metadata(root.join("config.cfg")).unwrap().permissions().mode();
    assert_eq!(file_perm & 0o777, 0o600, "File config harus kembali 0600");
}