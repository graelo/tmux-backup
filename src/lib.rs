#![warn(missing_docs)]

//! A backup & restore solution for Tmux sessions.
//!
//! This crate is consumed as a command-line binary rather than as a library API.
//!
//! End-user documentation — installation, usage, and configuration — lives in
//! the repository:
//!
//! - [README](https://github.com/graelo/tmux-backup#readme)

pub mod actions;
pub mod config;
pub mod error;
pub mod management;
pub use tmux_lib as tmux;

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, error::Error>;
