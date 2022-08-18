#![warn(missing_docs)]

//! A backup & restore solution for Tmux sessions.
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
//! - Does not handle multiple clients: help is welcome if you have clear scenarios for this.
//! - Does not handle session groups: help is also welcome.
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

pub mod actions;
pub mod config;
pub mod error;
pub mod management;
pub mod tmux;
