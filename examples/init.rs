//! # neuxcfg Initialisation Demo
//!
//! This example demonstrates the core initialisation process of the neuxcfg
//! library. It shows how to create a `Neuxcfg` instance, call `init()` to
//! set up the platform‑appropriate configuration directory, inspect the
//! resulting file structure, verify idempotency, and use a custom root path
//! for testing or alternative setups.
//!
//! ## Running
//!
//! ```bash
//! cargo run --example init
//! ```
//!
//! ## What to expect
//!
//! - The default config root will be printed (e.g. `~/.config/neuxcfg` on Linux,
//!   `~/Library/Application Support/neuxcfg` on macOS, etc.).
//! - After `init()`, the directory and the global `config.cfg` file are created
//!   (if they didn't already exist). Their existence and contents are shown.
//! - A second call to `init()` verifies that the operation is idempotent and does
//!   not corrupt existing data.
//! - A custom temporary directory is used to demonstrate `with_root()` without
//!   affecting your actual configuration.

use neuxcfg::Neuxcfg;

fn main() {
    println!("=== neuxcfg v{} Demo ===\n", env!("CARGO_PKG_VERSION"));

    // -------------------------------------------------------------------------
    // 1. Default configuration root
    // -------------------------------------------------------------------------
    match Neuxcfg::new() {
        Ok(cfg) => {
            println!(" Default config root: {:?}\n", cfg.root());

            // 1a. First-time initialisation
            match cfg.init() {
                Ok(()) => println!(" Initialisation successful!\n"),
                Err(e) => {
                    eprintln!(" Initialisation failed: {e}");
                    std::process::exit(1);
                }
            }

            // 1b. Inspect the created structure
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

            // 1c. Idempotency check – calling init() again must not fail
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

    // -------------------------------------------------------------------------
    // 2. Custom root via `with_root()` – useful for testing or isolation
    // -------------------------------------------------------------------------
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
