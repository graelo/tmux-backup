#![warn(missing_docs)]

//! # tmux-revive
//!
//! Save and restore your Tmux sessions.

pub mod config;
mod error;

pub mod management;

pub mod actions;

mod tmux;
pub use tmux::tmux_display_message;
