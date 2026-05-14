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
//! 5. Demonstrate a custom root with `with_root()` and **leave it on disk**
//!    so you can inspect the result.

use neuxcfg::Neuxcfg;

fn main() {
    println!("=== neuxcfg v{} Demo ===\n", env!("CARGO_PKG_VERSION"));

    // ── Default config directory (real location) ──────────────────────
    match Neuxcfg::new() {
        Ok(cfg) => {
            println!(" Default config root: {:?}\n", cfg.root());

            // Initialise (create directories & default file if missing)
            match cfg.init() {
                Ok(()) => println!(" Initialisation successful!\n"),
                Err(e) => {
                    eprintln!(" Initialisation failed: {e}");
                    std::process::exit(1);
                }
            }

            // Inspect root directory
            let root = cfg.root();
            println!("Root directory:");
            println!("   exists : {}", root.exists());
            println!("   is_dir : {}", root.is_dir());
            println!("   path   : {:?}", root);

            // Inspect config.cfg
            let config_file = root.join("config.cfg");
            println!("\nMain config file:");
            println!("   exists : {}", config_file.exists());
            println!("   is_file: {}", config_file.is_file());
            println!("   path   : {:?}", config_file);

            // Read and display content
            match std::fs::read_to_string(&config_file) {
                Ok(contents) => {
                    let shown = if contents.is_empty() {
                        "(empty)"
                    } else {
                        &contents
                    };
                    println!("   content: \"{}\"", shown.trim_end());
                }
                Err(e) => eprintln!("   Could not read file: {e}"),
            }

            // Idempotence demonstration
            println!("\n Calling init() a second time (idempotent test)...");
            if let Err(e) = cfg.init() {
                eprintln!("    Unexpected error: {e}");
            } else {
                println!("    No error, structure remains safe.");
            }
        }
        Err(e) => {
            eprintln!(" Could not create Neuxcfg: {e}");
            eprintln!(
                "Make sure your system supports `dirs::config_dir()` \
                 (e.g. $HOME or %APPDATA% is set)."
            );
            std::process::exit(2);
        }
    }

    // ── Custom root (temporary) ───────────────────────────────────────
    println!("\n Demonstrating with_root() using a temporary directory:");
    let custom_root = std::env::temp_dir().join("neuxcfg_demo_custom");
    let custom_cfg = Neuxcfg::with_root(custom_root.clone());
    match custom_cfg.init() {
        Ok(()) => {
            println!("    Custom root: {:?}", custom_cfg.root());
            let cfg_file = custom_cfg.root().join("config.cfg");
            println!("   config.cfg exists: {}", cfg_file.exists());
            if let Ok(contents) = std::fs::read_to_string(&cfg_file) {
                let shown = if contents.is_empty() {
                    "(empty)"
                } else {
                    &contents
                };
                println!("   content: \"{}\"", shown.trim_end());
            }
            //  Do NOT delete the directory – keep it for manual inspection.
            println!(
                "\n    Temporary directory kept for inspection: {:?}",
                custom_root
            );
            println!("   You can delete it manually when done.");
        }
        Err(e) => eprintln!("    Failed to initialise custom root: {e}"),
    }

    println!("\n Demo finished.");
}
