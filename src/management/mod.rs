//! Manage existing backup files.

pub mod catalog;
pub use catalog::Catalog;

pub mod compaction;
pub use compaction::{Plan, Strategy};
