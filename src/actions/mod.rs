//! Actions such as save and restoring a backup.

mod autosave;
pub use autosave::{
    AutosaveContext, autosave, context as autosave_context,
    display_message as display_autosave_message,
};
mod restore;
pub use restore::restore;
mod save;
pub use save::save;
