//! # neuxcfg Project Management Demo
//!
//! This example walks through the complete lifecycle of a single project:
//! creating a project, reading its default configuration, modifying it with
//! custom key‑value pairs, writing it back, and finally re‑reading to confirm
//! persistence. The project name used is `age-authenticator`, but any valid
//! name will work.
//!
//! ## Running
//!
//! ```bash
//! cargo run --example project
//! ```
//!
//! ## Behaviour
//!
//! - The default root is initialised (if not already present).
//! - A project directory is created under `<root>/age-authenticator/` with
//!   a `<project>.config.cfg` file containing the default `ProjectConfig`.
//! - Two extra fields (`secret_key` and `port`) are added to the project’s
//!   `extra` map.
//! - The updated config is saved and then reloaded, showing the final TOML
//!   content both as a debug print and as raw file content.

use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------------------------------------------
    // 1. Initialise neuxcfg
    // ------------------------------------------------------------------
    let cfg = Neuxcfg::new()?;
    cfg.init()?;
    println!("Root: {:?}\n", cfg.root());

    // ------------------------------------------------------------------
    // 2. Create a project
    // ------------------------------------------------------------------
    let project_name = "age-authenticator";
    match cfg.add_project(project_name) {
        Ok(()) => println!("Project '{}' created.\n", project_name),
        Err(e) => {
            eprintln!("Could not create project '{}': {e}", project_name);
            std::process::exit(1);
        }
    }

    // ------------------------------------------------------------------
    // 3. Read the default configuration (just name & path)
    // ------------------------------------------------------------------
    let mut config = cfg.get_project_config(project_name)?;
    println!("Default config:\n{:#?}\n", config);

    // ------------------------------------------------------------------
    // 4. Add custom fields to `extra`
    // ------------------------------------------------------------------
    config
        .project
        .extra
        .insert("secret_key".into(), toml::Value::String("s3cr3t".into()));
    config
        .project
        .extra
        .insert("port".into(), toml::Value::Integer(8080));

    // ------------------------------------------------------------------
    // 5. Write updated configuration back
    // ------------------------------------------------------------------
    cfg.set_project_config(project_name, &config)?;
    println!("Added custom fields.\n");

    // ------------------------------------------------------------------
    // 6. Re-read to confirm persistence
    // ------------------------------------------------------------------
    let reread = cfg.get_project_config(project_name)?;
    println!("Final config:\n{:#?}", reread);

    // Show the exact file content on disk
    let config_file = cfg
        .root()
        .join(project_name)
        .join(format!("{}.config.cfg", project_name));
    println!("\nConfig file: {:?}", config_file);
    println!("Content:");
    println!("{}", std::fs::read_to_string(&config_file)?);

    Ok(())
}
