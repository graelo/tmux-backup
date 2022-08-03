#![warn(missing_docs)]

//! # tmux-revive
//!

pub mod config;
mod error;
pub mod save;
mod tmux;
pub use tmux::display::display_message;

// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Catalog {
    sessions: Vec<tmux::session::Session>,
    windows: Vec<tmux::window::Window>,
}

/// Name of the directory storing the panes content in the archive.
///
/// This name is also used in the temporary directory when retrieving the panes content from Tmux.
const PANES_DIR_NAME: &str = "panes-content";

/// Name of the file storing the catalog in the archive.
///
/// This name is also used in the temporary directory when storing the catalog.
const CATALOG_FILENAME: &str = "catalog.yaml";

/// Report the number of sessions, windows and panes.
///
/// This report is displayed after the commands `save`, `restore`, or `describe`.
#[derive(Debug)]
pub struct Report {
    /// Number of sessions in an archive.
    pub num_sessions: u16,

    /// Number of windows in an archive.
    pub num_windows: u16,

    /// Number of panes in an archive.
    pub num_panes: u16,
}
