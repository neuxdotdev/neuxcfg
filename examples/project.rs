//! Demo: use `neuxcfg` to manage the configuration of a real project
//! (here: "age-authenticator").
//!
//! Run with:
//! ```bash
//! cargo run --example project
//! ```
//!
//! What it does:
//! 1. Initialises the global `neuxcfg` directory (if not already done).
//! 2. Lists all projects that already exist.
//! 3. Adds a new project called `age-authenticator`.
//! 4. Prints the default config (auto‑filled name and path).
//! 5. Updates the path to a custom location.
//! 6. Leaves all directories and files on disk for manual inspection.

use neuxcfg::Neuxcfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== neuxcfg Project Demo ===\n");

    // Use the default system config directory.
    let cfg = Neuxcfg::new()?;
    cfg.init()?;
    println!("Root: {:?}\n", cfg.root());

    // List existing projects before adding anything.
    let existing = cfg.list_projects()?;
    println!("Projects before: {:?}\n", existing);

    // Add our new project "age-authenticator".
    let project_name = "age-authenticator";
    match cfg.add_project(project_name) {
        Ok(()) => println!(" Project '{}' created.\n", project_name),
        Err(e) => {
            eprintln!(" Could not create project '{}': {e}", project_name);
            std::process::exit(1);
        }
    }

    // Read and display the default configuration.
    let config = cfg.get_project_config(project_name)?;
    println!("Default config of '{}':", project_name);
    println!("  name: {}", config.project.name);
    println!("  path: {}\n", config.project.path);

    // Update the project's path to something custom.
    let new_path = "/opt/age-authenticator/data";
    let mut updated_config = config;
    updated_config.project.path = new_path.into();
    cfg.set_project_config(project_name, &updated_config)?;
    println!(" Updated path to: {}\n", new_path);

    // Read back to confirm.
    let reread = cfg.get_project_config(project_name)?;
    println!("Re-read config:");
    println!("  name: {}", reread.project.name);
    println!("  path: {}\n", reread.project.path);

    // List projects again.
    let after = cfg.list_projects()?;
    println!("Projects after: {:?}", after);

    // Show the actual file location.
    let config_file = cfg
        .root()
        .join(project_name)
        .join(format!("{}.config.cfg", project_name));
    println!("\n Config file on disk: {:?}", config_file);
    println!("You can inspect it with: cat {:?}", config_file);

    Ok(())
}