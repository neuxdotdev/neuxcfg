//! Demo: initialise and verify the `neuxcfg` configuration directory structure.
//!
//! Run with:
//! ```bash
//! cargo run --example init
//! ```
//!
//! This program will:
//! 1. Create the `<config_dir>/neuxcfg` directory and `config.cfg` file
//!    (if they do not exist yet)
//! 2. Print the root path
//! 3. Confirm that initialisation is safe to call multiple times (idempotent)
//! 4. Handle errors when the system config directory cannot be determined
//!    (e.g. in a minimal environment)

use neuxcfg::Neuxcfg;

fn main() {
    println!("=== neuxcfg v0.1.0 Demo ===\n");

    // Create a Neuxcfg instance pointing to the default system config directory.
    match Neuxcfg::new() {
        Ok(cfg) => {
            println!("Configuration root will be stored at: {:?}\n", cfg.root());

            // Perform initialisation: create directories and the default file.
            match cfg.init() {
                Ok(()) => println!("Initialisation succeeded!\n"),
                Err(e) => {
                    eprintln!("Initialisation failed: {e}");
                    std::process::exit(1);
                }
            }

            // Display status of the root directory and the main config file.
            let root = cfg.root();
            println!("Root directory:");
            println!("   exists : {}", root.exists());
            println!("   is_dir : {}", root.is_dir());
            println!("   path   : {:?}", root);

            let config_file = root.join("config.cfg");
            println!("\nMain config file:");
            println!("   exists : {}", config_file.exists());
            println!("   is_file: {}", config_file.is_file());
            println!("   path   : {:?}", config_file);

            // Read the content of the configuration file (expected to be empty).
            if let Ok(contents) = std::fs::read_to_string(&config_file) {
                let empty_note = if contents.is_empty() { " (empty)" } else { "" };
                println!("   content: \"{}\"{}", contents, empty_note);
            }

            // Demonstrate idempotence: calling init a second time must succeed.
            println!("\nCalling init() a second time (idempotent)...");
            if let Err(e) = cfg.init() {
                eprintln!("   Unexpected error: {e}");
            } else {
                println!("   No error, structure remains safe.");
            }

            // Demonstrate with_root with a custom (temporary) directory.
            println!("\nDemonstrating with_root() using a temporary directory:");
            let custom_root = std::env::temp_dir().join("neuxcfg_demo_custom");
            let custom_cfg = Neuxcfg::with_root(custom_root.clone());
            if let Err(e) = custom_cfg.init() {
                eprintln!("   Failed to initialise custom root: {e}");
            } else {
                println!("   Custom root: {:?}", custom_cfg.root());
                println!(
                    "   config.cfg exists: {}",
                    custom_cfg.root().join("config.cfg").exists()
                );
                // Clean up the demo directory.
                let _ = std::fs::remove_dir_all(&custom_root);
                println!("   Temporary directory removed.");
            }

            println!("\nDemo finished.");
        }
        Err(e) => {
            eprintln!("Could not create Neuxcfg: {e}");
            eprintln!(
                "Make sure your system supports `dirs::config_dir()` (e.g. $HOME or %APPDATA% is set)."
            );
            std::process::exit(2);
        }
    }
}
