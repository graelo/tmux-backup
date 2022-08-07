//! [![crate](https://img.shields.io/crates/v/tmux-revive.svg)](https://crates.io/crates/tmux-revive)
//! [![documentation](https://docs.rs/tmux-revive/badge.svg)](https://docs.rs/tmux-revive)
//! [![minimum rustc 1.8](https://img.shields.io/badge/rustc-1.50+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
//! [![build status](https://github.com/graelo/tmux-revive/workflows/main/badge.svg)](https://github.com/graelo/tmux-revive/actions)
//!
//! Format value with units according to SI ([système international d'unités](https://en.wikipedia.org/wiki/International_System_of_Units)).
//!
//! _Version requirement: rustc 1.50+_
//!
//! ```toml
//! [dependencies]
//! tmux-revive = "0.1"
//! ```
//!
//! ## Getting started
//!
//!
//!
//! ## License
//!
//! Licensed under either of
//!
//!  * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
//!  * [MIT license](http://opensource.org/licenses/MIT)
//!
//! at your option.
//!
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall
//! be dual licensed as above, without any additional terms or conditions.

pub mod config;
mod error;

pub mod management;

pub mod actions;

mod tmux;
pub use tmux::tmux_display_message;
