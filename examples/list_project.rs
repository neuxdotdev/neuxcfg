// examples/list_project.rs
//
// Demonstrates how to discover all managed projects using the
// `list_projects()` method and inspect their configuration.
//
// Run with:
// ```bash
// cargo run --example list_project
// ```

use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the default system configuration directory.
    let cfg = Neuxcfg::new()?;
    cfg.init()?;

    println!("=== neuxcfg – Project Listing Demo ===\n");
    println!("Root: {:?}\n", cfg.root());

    // Retrieve all project names currently stored.
    let projects = cfg.list_projects()?;

    if projects.is_empty() {
        println!("No projects found. Create one with `cargo run --example project`.");
        return Ok(());
    }

    println!("Found {} project(s):\n", projects.len());

    for name in &projects {
        println!("  • {name}");

        // Read and display the project's configuration.
        match cfg.get_project_config(name) {
            Ok(config) => {
                println!("    name : {}", config.project.name);
                println!("    path : {}", config.project.path);

                if !config.project.extra.is_empty() {
                    println!("    extra:");
                    for (key, value) in &config.project.extra {
                        println!("      {key} = {value}");
                    }
                }
            }
            Err(e) => {
                eprintln!("    Failed to read config for '{name}': {e}");
            }
        }
        println!();
    }

    println!("Use `cfg.list_projects()` in your own code to obtain this list dynamically.");
    Ok(())
}