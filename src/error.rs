//! Error types for the `neuxcfg` library.
//!
//! This module defines [`NeuxcfgError`], a small, ergonomic error enum that
//! covers all failure cases the library can encounter. It implements
//! [`std::error::Error`] via `thiserror`, as well as [`PartialEq`] for easy
//! comparison in tests.
//!
//! # Design notes
//!
//! The `Io` variant stores the I/O error as a `String` rather than
//! wrapping [`std::io::Error`] directly. This decision was made to allow
//! `NeuxcfgError` to derive `PartialEq` (which `std::io::Error` does not
//! implement) and to keep the public API free of `std::io::Error` in its
//! type signature. The original error message is preserved for debugging,
//! and the conversion is lossless from a human‑readable perspective.

use thiserror::Error;

/// Represents all possible errors that can occur in the `neuxcfg` library.
///
/// This type is returned by [`Neuxcfg::new`](crate::Neuxcfg::new) and
/// [`Neuxcfg::init`](crate::Neuxcfg::init).
///
/// # Examples
///
/// ```rust
/// use neuxcfg::NeuxcfgError;
///
/// // ConfigDirNotFound can be constructed manually for testing.
/// let err = NeuxcfgError::ConfigDirNotFound;
/// assert_eq!(err.to_string(), "cannot determine config directory");
/// ```
///
/// ```rust
/// use neuxcfg::NeuxcfgError;
///
/// // I/O errors are converted from std::io::Error.
/// let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
/// let cfg_err: NeuxcfgError = io_err.into();
/// assert!(cfg_err.to_string().contains("I/O error"));
/// ```
#[derive(Error, Debug, PartialEq)]
pub enum NeuxcfgError {
    /// The system configuration directory could not be determined.
    ///
    /// This typically means the `dirs` crate returned `None`, which can
    /// happen on platforms without a home directory or if the environment
    /// variables (`XDG_CONFIG_HOME`, `HOME`) are missing.
    #[error("cannot determine config directory")]
    ConfigDirNotFound,

    /// An I/O error occurred while creating directories or files.
    ///
    /// The original error message is stored as a `String` to allow
    /// `PartialEq` comparisons. Use the `Display` implementation or the
    /// `{0}` format specifier to see the full message.
    #[error("I/O error: {0}")]
    Io(String),
}

/// Enables automatic conversion from [`std::io::Error`] to [`NeuxcfgError`].
///
/// This allows the use of the `?` operator on `std::io::Result` inside
/// functions that return `Result<_, NeuxcfgError>`. The original error
/// message is preserved via its `Display` implementation.
///
/// # Example
///
/// ```rust
/// use neuxcfg::NeuxcfgError;
///
/// fn example() -> Result<(), NeuxcfgError> {
///     // This `?` will convert the io::Error into NeuxcfgError::Io.
///     std::fs::read_to_string("/nonexistent")?;
///     Ok(())
/// }
/// ```
impl From<std::io::Error> for NeuxcfgError {
    fn from(err: std::io::Error) -> Self {
        NeuxcfgError::Io(err.to_string())
    }
}
