//! Vendored dependencies.
//!
//! The original package vendors its two upstream dependencies rather than
//! depending on them, so that the published library has no dependencies of its
//! own. That decision is preserved here: these modules are part of the code
//! being maintained, not external crates.

pub mod ansi_styles;
pub mod supports_color;
