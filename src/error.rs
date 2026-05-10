/// Errors when interacting with metadata refs in a Git repository.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("metadata not found for object {0}")]
    NotFound(gix::ObjectId),
    #[error("metadata has invalid type {0}")]
    InvalidType(gix::object::Kind),
    #[error("hash kind {0} is not supported yet")]
    UnsupportedHashKind(gix::ObjectId, gix::hash::Kind),
}
