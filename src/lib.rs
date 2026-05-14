//! # neuxcfg
//!
//! A tiny, security‑conscious library for managing an application’s
//! configuration directory.
//!
//! `neuxcfg` creates a dedicated configuration folder (`neuxcfg`) inside the
//! user’s system‑standard configuration directory (e.g.,
//! `~/.config/neuxcfg` on Linux, `~/Library/Application Support/neuxcfg`
//! on macOS, or `C:\Users\<User>\AppData\Roaming\neuxcfg` on Windows) and
//! ensures that only the owning user can read or write its contents on
//! Unix‑like systems.
//!
//! ## Features
//!
//! - **Automatic path resolution** – Uses the [`dirs`] crate to locate the
//!   correct configuration directory for the current platform.
//! - **Idempotent initialisation** – Calling [`init`](Neuxcfg::init) more than
//!   once is safe; existing files and permissions are preserved, while missing
//!   ones are created.
//! - **Unix permission hardening** – On Unix, the root directory is set to
//!   `0o700` (owner‑only access) and the `config.cfg` file is set to `0o600`
//!   (owner read/write). Existing files are re‑permissioned to `0o600` in case
//!   they were previously left with weaker permissions.
//! - **Custom root paths** – If the default location is not suitable, an
//!   arbitrary root directory can be supplied via
//!   [`with_root`](Neuxcfg::with_root).
//! - **Clear error handling** – All errors are mapped to the
//!   [`NeuxcfgError`](error::NeuxcfgError) type, which implements `Display`,
//!   `Debug`, and `PartialEq`.
//!
//! ## Security Model
//!
//! The library is designed for applications that store sensitive data in
//! their configuration directory (e.g., API keys, database passwords, tokens).
//! By default, on Unix systems:
//!
//! - The root directory (`…/neuxcfg`) is created with mode `0700`. This means
//!   only the owner can list its contents or enter the directory. Group and
//!   others have no access at all.
//! - The main configuration file (`config.cfg`) is created with mode `0600`,
//!   ensuring only the owner can read or write it. Even if the directory
//!   permissions were to become looser later, the file itself remains
//!   protected.
//!
//! On Windows these permission calls are **no‑ops**; Windows has a different
//! security model (ACLs) and is typically considered secure enough for this
//! use‑case without extra steps. The library concentrates Unix hardening
//! behind `#[cfg(unix)]` so that no misleading code is executed on other
//! platforms.
//!
//! ## Cross‑Platform Behaviour
//!
//! | Aspect                     | Linux / macOS                      | Windows                            |
//! |----------------------------|------------------------------------|------------------------------------|
//! | Config directory source    | `dirs::config_dir()` (XDG / macOS) | `dirs::config_dir()` (AppData)     |
//! | Directory permissions      | Set to `0o700`                     | No change                          |
//! | File permissions           | Set to `0o600`                     | No change                          |
//! | `create_new(true)`         | Atomic creation, fails if exists   | Atomic creation, fails if exists   |
//!
//! ## Usage
//!
//! Add `neuxcfg` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! neuxcfg = "0.1.0"   # hypothetical version
//! ```
//!
//! Then, in your application:
//!
//! ```rust,no_run
//! use neuxcfg::Neuxcfg;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Use the default system config directory.
//!     let cfg = Neuxcfg::new()?;
//!
//!     // Initialise (safe to call many times).
//!     cfg.init()?;
//!
//!     println!("Configuration root: {:?}", cfg.root());
//!     Ok(())
//! }
//! ```
//!
//! If you need a custom location, for example during testing:
//!
//! ```rust,no_run
//! use neuxcfg::Neuxcfg;
//! use std::path::PathBuf;
//!
//! let temp_dir = std::env::temp_dir().join("neuxcfg_test");
//! let cfg = Neuxcfg::with_root(temp_dir);
//! cfg.init().expect("init should succeed in a temp dir");
//! ```

mod error;
pub use error::NeuxcfgError;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The central type of the library, holding the path to the configuration
/// root directory.
///
/// After obtaining an instance (either via [`Neuxcfg::new`] or
/// [`Neuxcfg::with_root`]), you normally call [`init`](Neuxcfg::init) once
/// at program startup. The instance can then be used to retrieve the root
/// path and construct paths to other configuration files.
///
/// # Examples
///
/// ```rust,no_run
/// # use neuxcfg::Neuxcfg;
/// let cfg = Neuxcfg::new().expect("Could not determine config directory");
/// cfg.init().expect("Initialisation failed");
/// assert!(cfg.root().join("config.cfg").exists());
/// ```
pub struct Neuxcfg {
    /// The absolute path to the configuration root.
    ///
    /// This is always the directory that will contain `config.cfg` and any
    /// other application‑specific files. It is **not** created until
    /// [`init`](Neuxcfg::init) is called.
    root: PathBuf,
}

impl Neuxcfg {
    /// Creates a new `Neuxcfg` instance pointing to the default system
    /// configuration directory.
    ///
    /// The path is obtained by calling [`dirs::config_dir()`] and appending
    /// `"neuxcfg"`. If the system cannot determine a configuration directory
    /// (e.g., on a minimal Linux system without `$HOME`), a
    /// [`NeuxcfgError::ConfigDirNotFound`] error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ConfigDirNotFound`] when the underlying
    /// platform reports no config directory.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use neuxcfg::Neuxcfg;
    /// match Neuxcfg::new() {
    ///     Ok(cfg) => println!("Config will be stored at {:?}", cfg.root()),
    ///     Err(e) => eprintln!("Cannot determine config directory: {e}"),
    /// }
    /// ```
    pub fn new() -> Result<Self, NeuxcfgError> {
        let config_dir = dirs::config_dir().ok_or(NeuxcfgError::ConfigDirNotFound)?;
        Ok(Self {
            root: config_dir.join("neuxcfg"),
        })
    }

    /// Creates a `Neuxcfg` instance with an arbitrary root directory.
    ///
    /// This constructor does **not** check whether the path is valid or
    /// writable; those checks are deferred to [`init`](Neuxcfg::init).
    /// Use this variant when you want to store configuration in a
    /// non‑standard location (e.g., a test temporary directory).
    ///
    /// # Parameters
    ///
    /// * `root` – The absolute or relative path that will serve as the
    ///   configuration root. No normalisation is applied.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use neuxcfg::Neuxcfg;
    /// let custom = Neuxcfg::with_root("/opt/myapp/config".into());
    /// custom.init().expect("Must be able to write to /opt/myapp/config");
    /// ```
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Initialises the configuration directory, creating it and the default
    /// `config.cfg` file if they do not exist, and hardening Unix permissions.
    ///
    /// This method is **idempotent**: calling it multiple times will not
    /// overwrite an existing `config.cfg`, and will re‑apply the correct
    /// Unix permissions even if the file was already present (in case they
    /// were altered).
    ///
    /// # What it does
    ///
    /// 1. Creates the root directory (and all missing parents) using
    ///    [`std::fs::create_dir_all`].
    /// 2. On Unix, sets the root directory permissions to `0o700`.
    /// 3. Attempts to create `config.cfg` inside the root directory **only**
    ///    if it does not already exist ([`OpenOptions::create_new(true)`]).
    ///    - If the file is created successfully, Unix permissions are set
    ///      to `0o600`.
    ///    - If the file already exists (`ErrorKind::AlreadyExists`), the
    ///      method **still** sets its permissions to `0o600` on Unix to
    ///      ensure they are secure. Other I/O errors are propagated.
    ///
    /// # Errors
    ///
    /// Returns an [`NeuxcfgError`] if:
    /// - The root directory could not be created (permissions, invalid path,
    ///   etc.).
    /// - On Unix, `set_permissions` fails for the directory or the file.
    /// - The `config.cfg` file cannot be opened or created for any reason
    ///   other than `AlreadyExists`.
    ///
    /// # Platform behaviour
    ///
    /// On non‑Unix platforms the permission‑setting blocks are compiled out,
    /// so no permission changes occur. File creation is handled identically.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use neuxcfg::Neuxcfg;
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;                       // first call: creates everything
    /// cfg.init()?;                       // second call: no-op, safe
    /// assert!(cfg.root().join("config.cfg").exists());
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn init(&self) -> Result<(), NeuxcfgError> {
        // 1. Ensure the root directory exists.
        std::fs::create_dir_all(&self.root)?;

        // 2. Harden root directory permissions (Unix only).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.root, std::fs::Permissions::from_mode(0o700))?;
        }

        let config_path = self.root.join("config.cfg");

        // 3. Create the default config file only if it doesn't exist.
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true) // fails with AlreadyExists if the file is present
            .open(&config_path)
        {
        Ok(mut file) => {
            // New file created – write the current library version into it.
            let version = env!("CARGO_PKG_VERSION");
            let content = format!("version = \"{}\"\n", version);
            std::io::Write::write_all(&mut file, content.as_bytes())?;
            // Harden permissions after writing (Unix only).
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
            }
        }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                // File already exists – still re-apply permissions in case
                // they were tampered with (Unix only).
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
                }
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    /// Returns a reference to the configuration root path.
    ///
    /// The path is **not** guaranteed to exist; use [`init`](Neuxcfg::init)
    /// to create the directory structure before reading or writing files
    /// inside it.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use neuxcfg::Neuxcfg;
    /// let cfg = Neuxcfg::with_root("/tmp/myapp".into());
    /// assert_eq!(cfg.root(), std::path::Path::new("/tmp/myapp"));
    /// ```
    pub fn root(&self) -> &Path {
        &self.root
    }
}
