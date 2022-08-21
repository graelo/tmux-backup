//! Functions to read or manipulate Tmux

pub mod error;

pub mod client;
pub use client::display_message;
pub mod layout;
pub mod pane;
pub mod pane_id;
pub mod server;
pub mod session;
pub mod session_id;
pub mod window;
pub mod window_id;
