//! Shared helpers for `find_metadata` tests. Re-exports the `metadata`
//! helpers (which themselves re-export the `metadatas` helpers).

#[path = "../metadata/helpers.rs"]
#[allow(dead_code)]
mod metadata_helpers;

pub use metadata_helpers::*;
