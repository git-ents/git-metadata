use crate::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Metadata {
    /// The metadata entry's unique identifier which serves as the fanout key.
    ///
    /// The [`ObjectId`] is guaranteed to be unique within its repository at the
    /// time of the [`new`] function call. Deleting the object from the repository
    /// before the [`Metadata`] instance is dropped is considered a logic ([TOCTOU])
    /// error.
    ///
    /// [`ObjectId`]: gix::ObjectId
    /// [`new`]: Metadata::new
    /// [TOCTOU]: https://en.wikipedia.org/wiki/Time-of-check_to_time-of-use
    id: gix::ObjectId,
    /// The metadata content stored as a tree object in the repository.
    ///
    /// The [`ObjectId`] references a tree that must remain reachable within its
    /// repository for the lifetime of the [`Metadata`] instance. Removing the
    /// tree (e.g. via garbage collection) before the instance is dropped is
    /// considered a [logic] error.
    ///
    /// [`ObjectId`]: gix::ObjectId
    /// [logic]: https://en.wikipedia.org/wiki/Time-of-check_to_time-of-use
    data: gix::ObjectId,
}

impl Metadata {
    /// Creates a new [`Metadata`] instance after verifying that `id` references a
    /// blob and `data` references a tree in the given repository.
    ///
    /// Checks are performed in argument order; the first failure short-circuits.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedHashKind`] if either `id` or `data` uses a
    /// hash kind other than SHA-1. Returns [`Error::NotFound`] if either object
    /// does not exist in the repository. Returns [`Error::InvalidType`] if `id`
    /// is not a blob or `data` is not a tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use git_metadata::Metadata;
    ///
    /// let dir = tempfile::tempdir().expect("tempdir");
    /// let repo = gix::init(dir.path()).expect("init repository");
    /// let id = repo.write_blob(b"hello").expect("write blob").detach();
    /// let data = gix::ObjectId::empty_tree(gix::hash::Kind::Sha1);
    /// let metadata = Metadata::new(&repo, id, data)?;
    /// # Ok::<(), git_metadata::Error>(())
    /// ```
    pub fn new(
        repo: &gix::Repository,
        id: gix::ObjectId,
        data: gix::ObjectId,
    ) -> Result<Self, Error> {
        Ok(Self {
            id: verify(repo, id, gix::object::Kind::Blob)?,
            data: verify(repo, data, gix::object::Kind::Tree)?,
        })
    }
}

fn verify(
    repo: &gix::Repository,
    oid: gix::ObjectId,
    expected: gix::object::Kind,
) -> Result<gix::ObjectId, Error> {
    if !matches!(oid, gix::ObjectId::Sha1(_)) {
        return Err(Error::UnsupportedHashKind(oid, oid.kind()));
    }
    let kind = repo
        .try_find_header(oid)
        .ok()
        .flatten()
        .ok_or(Error::NotFound(oid))?
        .kind();
    if kind != expected {
        return Err(Error::InvalidType(kind));
    }
    Ok(oid)
}
