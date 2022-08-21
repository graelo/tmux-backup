//! Functions to read or manipulate Tmux
//!
//! Version requirement: _rustc 1.50+_
//!
//! ```toml
//! [dependencies]
//! tmux-backup = "0.1"
//! ```
//!
//! ## Getting started
//!
//! Work in progress
//!
//! ## Caveats
//!
//! - This is a beta version
//!
//! ## License
//!
//! Licensed under either of
//!
//! - [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
//! - [MIT license](http://opensource.org/licenses/MIT)
//!
//! at your option.
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall
//! be dual licensed as above, without any additional terms or conditions.

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
