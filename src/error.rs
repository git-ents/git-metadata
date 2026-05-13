/// Errors when interacting with metadata refs in a Git repository.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("metadata not found for object {0}")]
    NotFound(gix::ObjectId),
    #[error("metadata has invalid type {0}")]
    InvalidType(gix::object::Kind),
    #[error("hash kind {0} is not supported yet")]
    UnsupportedHashKind(gix::ObjectId, gix::hash::Kind),
    #[error("invalid `.fanout` depth {value:?}; must be a decimal integer in 1..=19")]
    InvalidFanoutDepth { value: gix::bstr::BString },
    #[error("invalid `.fanout` entry type `{kind}`; must be a blob")]
    InvalidFanoutType { kind: String },
    #[error("metadata already exists for object {0}")]
    AlreadyExists(gix::ObjectId),
    #[error("invalid root object type {0}; must be a tree or commit")]
    InvalidRootType(gix::object::Kind),
    #[error("fanout path conflicts with existing non-tree entry {0:?}")]
    FanoutPathConflict(gix::bstr::BString),
    /// Any underlying `gix` failure that we don't classify domain-specifically.
    /// Downcast via [`std::error::Error::source`] for the original error.
    #[error(transparent)]
    Gix(Box<dyn std::error::Error + Send + Sync + 'static>),
}

macro_rules! impl_gix_from {
    ($($ty:path),* $(,)?) => {
        $(
            impl From<$ty> for Error {
                fn from(e: $ty) -> Self {
                    Error::Gix(Box::new(e))
                }
            }
        )*
    };
}

impl_gix_from! {
    gix::object::find::existing::with_conversion::Error,
    gix::reference::find::existing::Error,
    gix::reference::find::Error,
    gix::reference::peel::Error,
    gix::reference::peel::to_kind::Error,
    gix::traverse::tree::breadthfirst::Error,
    gix::object::find::existing::Error,
    gix::object::find::Error,
    gix::objs::decode::Error,
    gix::object::write::Error,
    gix::commit::Error,
    gix::reference::edit::Error,
}
