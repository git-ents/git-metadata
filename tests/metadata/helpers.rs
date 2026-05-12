//! Shared helpers for `metadata` tests. Re-exports the `metadatas` helpers and
//! adds a deterministic signature factory.

#[path = "../metadatas/helpers.rs"]
mod inner;

pub use inner::*;

use gix::actor::SignatureRef;
use gix::bstr::BStr;

pub fn sig() -> SignatureRef<'static> {
    SignatureRef {
        name: BStr::new(b"Tester"),
        email: BStr::new(b"tester@example.com"),
        time: "1700000000 +0000",
    }
}

/// Compute the fanout-leaf path segments for `id` at the given depth.
pub fn leaf_path(id: gix::ObjectId, depth: u8) -> Vec<gix::bstr::BString> {
    let hex = hex_of(id);
    fanout_segments(&hex, depth as usize)
}
