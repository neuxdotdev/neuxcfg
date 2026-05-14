use neuxcfg::Neuxcfg;

#[test]
fn test_add_list_remove_project() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    cfg.add_project("myapp").unwrap();
    assert!(cfg.project_exists("myapp").unwrap());
    let list = cfg.list_projects().unwrap();
    assert!(list.contains(&"myapp".to_string()));
    let config = cfg.get_project_config("myapp").unwrap();
    assert_eq!(config.project.name, "myapp");
    let expected_path = cfg.root().join("myapp");
    assert_eq!(config.project.path, expected_path.to_string_lossy());
    assert!(config.project.extra.is_empty());
    cfg.remove_project("myapp").unwrap();
    assert!(!cfg.project_exists("myapp").unwrap());
}
#[test]
fn test_add_existing_project_errors() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    cfg.add_project("dup").unwrap();
    let err = cfg.add_project("dup").unwrap_err();
    assert_eq!(
        err,
        neuxcfg::NeuxcfgError::ProjectAlreadyExists("dup".into())
    );
}
#[test]
fn test_invalid_project_name() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    assert!(cfg.add_project("../escape").is_err());
    assert!(cfg.add_project("valid_name").is_ok());
    assert!(cfg.add_project("with/slash").is_err());
    assert!(cfg.add_project("with\\backslash").is_err());
    assert!(cfg.add_project("").is_err());
}
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
#[test]
fn test_project_not_found_errors() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    assert_eq!(
        cfg.get_project_config("ghost").unwrap_err(),
        neuxcfg::NeuxcfgError::ProjectNotFound("ghost".into())
    );
    assert_eq!(
        cfg.remove_project("ghost").unwrap_err(),
        neuxcfg::NeuxcfgError::ProjectNotFound("ghost".into())
    );
}
#[test]
fn test_project_exists_invalid_name() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = Neuxcfg::with_root(tmp.path().to_path_buf());
    cfg.init().unwrap();
    assert!(cfg.project_exists("../bad").is_err());
}
