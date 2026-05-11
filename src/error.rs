/// Errors when interacting with metadata refs in a Git repository.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("metadata not found for object {0}")]
    NotFound(gix::ObjectId),
    #[error("metadata has invalid type {0}")]
    InvalidType(gix::object::Kind),
    #[error("hash kind {0} is not supported yet")]
    UnsupportedHashKind(gix::ObjectId, gix::hash::Kind),
    #[error("invalid fanout leaf at {path:?}")]
    InvalidFanoutLeaf { path: gix::bstr::BString },
    #[error("invalid `.fanout` depth {value:?}; must be a decimal integer in 1..=19")]
    InvalidFanoutDepth { value: gix::bstr::BString },
    #[error(transparent)]
    FanoutFind(#[from] gix::object::find::existing::with_conversion::Error),
    #[error(transparent)]
    Reference(#[from] gix::reference::find::existing::Error),
    #[error(transparent)]
    Peel(#[from] gix::reference::peel::to_kind::Error),
    #[error(transparent)]
    Traverse(#[from] gix::traverse::tree::breadthfirst::Error),
}
