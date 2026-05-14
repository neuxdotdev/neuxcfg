use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Neuxcfg::new()?;
    cfg.init()?;
    println!("Root: {:?}\n", cfg.root());
    let project_name = "age-authenticator";
    match cfg.add_project(project_name) {
        Ok(()) => println!("✅ Project '{}' created.\n", project_name),
        Err(e) => {
            eprintln!("❌ Could not create project '{}': {e}", project_name);
            std::process::exit(1);
        }
    }
    let mut config = cfg.get_project_config(project_name)?;
    println!("Default config:\n{:#?}\n", config);
    config
        .project
        .extra
        .insert("secret_key".into(), toml::Value::String("s3cr3t".into()));
    config
        .project
        .extra
        .insert("port".into(), toml::Value::Integer(8080));
    cfg.set_project_config(project_name, &config)?;
    println!("✅ Added custom fields.\n");
    let reread = cfg.get_project_config(project_name)?;
    println!("Final config:\n{:#?}", reread);
    let config_file = cfg
        .root()
        .join(project_name)
        .join(format!("{}.config.cfg", project_name));
    println!("\n📁 Config file: {:?}", config_file);
    println!("Content:");
    println!("{}", std::fs::read_to_string(&config_file)?);
    Ok(())
}
