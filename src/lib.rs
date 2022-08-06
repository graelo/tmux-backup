#![warn(missing_docs)]

//! # tmux-revive
//!
//! Save and restore your Tmux sessions.

pub mod config;
mod error;

pub mod management;

mod report;
pub use report::Report;

pub mod actions;

mod tmux;
pub use tmux::tmux_display_message;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Summary {
    sessions: Vec<tmux::session::Session>,
    windows: Vec<tmux::window::Window>,
}

/// Name of the directory storing the panes content in the backup.
///
/// This name is also used in the temporary directory when retrieving the panes content from Tmux.
const PANES_DIR_NAME: &str = "panes-content";

/// Name of the file storing the catalog in the backup.
///
/// This name is also used in the temporary directory when storing the catalog.
const SUMMARY_FILENAME: &str = "catalog.yaml";
