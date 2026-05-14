use neuxcfg::Neuxcfg;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    #[test]
    fn test_init_creates_structure() {
        let tmp = TempDir::new().unwrap();
        let cfg = Neuxcfg::with_root(tmp.path().join("neuxcfg"));
        cfg.init().unwrap();
        let root = cfg.root();
        assert!(root.exists());
        let config_file = root.join("config.cfg");
        assert!(config_file.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = config_file.metadata().unwrap();
            assert_eq!(meta.permissions().mode() & 0o777, 0o600);
            let meta_root = root.metadata().unwrap();
            assert_eq!(meta_root.permissions().mode() & 0o777, 0o700);
        }
    }
    #[test]
    fn test_init_idempotent() {
        let tmp = TempDir::new().unwrap();
        let cfg = Neuxcfg::with_root(tmp.path().join("neuxcfg"));
        cfg.init().unwrap();
        cfg.init().unwrap();
    }
    #[test]
    fn test_root_custom() {
        let tmp = TempDir::new().unwrap();
        let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
        cfg.init().unwrap();
        assert!(tmp.path().join("config.cfg").exists());
    }
}