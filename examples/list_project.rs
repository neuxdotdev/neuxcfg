//! # neuxcfg Project Listing Demo
//!
//! Demonstrates how to retrieve all projects managed by neuxcfg and inspect
//! their individual configurations. The `list_projects()` method returns
//! the names of all valid subdirectories in the root; this example iterates
//! over them and prints key details.
//!
//! ## Running
//!
//! ```bash
//! cargo run --example list_project
//! ```
//!
//! ## Prerequisites
//!
//! This example expects at least one project to already exist. If the root
//! contains no projects, a message will be displayed suggesting to run the
//! `project` example first.
//!
//! ## Output
//!
//! For each project, the example prints:
//! - The project name
//! - The path stored inside the configuration
//! - Any custom `extra` fields that were previously added
//!
//! The final line reminds you that `list_projects()` can be used in your own
//! code to dynamically obtain the list of managed projects.

use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------------------------------------------
    // 1. Initialise neuxcfg (creates root if needed)
    // ------------------------------------------------------------------
    let cfg = Neuxcfg::new()?;
    cfg.init()?;

    println!("=== neuxcfg – Project Listing Demo ===\n");
    println!("Root: {:?}\n", cfg.root());

    // ------------------------------------------------------------------
    // 2. Obtain the list of project names
    // ------------------------------------------------------------------
    let projects = cfg.list_projects()?;

    if projects.is_empty() {
        println!("No projects found. Create one with `cargo run --example project`.");
        return Ok(());
    }

    println!("Found {} project(s):\n", projects.len());

    // ------------------------------------------------------------------
    // 3. Iterate and display each project's configuration
    // ------------------------------------------------------------------
    for name in &projects {
        println!("  * {name}");

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
