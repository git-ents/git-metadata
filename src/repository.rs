use crate::{Error, Metadata};

/// Interact with metadata refs in a Git repository.
///
/// The [`MetadataRepository`] trait is implemented by [`gix::Repository`], but
/// mirrors the `git2::Repository` notes API. When `gix-notes` is designated
/// [usable] by the Gitoxide team, this trait will be updated (with a new major
/// version) to mirror the Gitoxide API.
///
/// # Fanout layout
///
/// Metadata is stored under a Git tree (the "fanout tree") whose entries are
/// laid out by recursive 2-byte hex prefixes of the target object's hash. The
/// depth — the number of 2-byte prefix levels — is read from a `.fanout` blob
/// at the root of the fanout tree, and must be a decimal integer in `1..=19`.
/// When the `.fanout` blob is absent, depth defaults to [`DEFAULT_FANOUT`]
/// (the git-notes shape: `ab/cdef…`).
///
/// [`DEFAULT_FANOUT`]: crate::DEFAULT_FANOUT
///
/// At depth `d`, each leaf path has exactly `d + 1` segments: `d` segments of
/// 2 hex chars each, followed by one segment of `40 − 2d` hex chars. Entries
/// that do not match this shape are silently skipped.
///
/// [usable]: https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md
pub trait MetadataRepository {
    fn metadata(
        &self,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
        metadatas_ref: Option<&str>,
        oid: gix::ObjectId,
        metadata: &gix::ObjectId,
        force: bool,
    ) -> Result<gix::ObjectId, Error>;

    /// Returns the default fanout ref name used when callers pass `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use git_metadata::MetadataRepository;
    ///
    /// let dir = tempfile::tempdir().expect("tempdir");
    /// let repo = gix::init(dir.path()).expect("init repository");
    /// assert_eq!(repo.metadata_default_ref()?, "refs/metadata/commits");
    /// # Ok::<(), git_metadata::Error>(())
    /// ```
    fn metadata_default_ref(&self) -> Result<String, Error>;

    /// Returns the fanout depth for the tree at `metadatas_ref`.
    ///
    /// When `metadatas_ref` is `None`, [`metadata_default_ref`] is used. The
    /// depth is read from a `.fanout` blob at the root of the tree, which
    /// must contain a decimal integer in `1..=19` whose two-hex-character
    /// segments leave at least one hex character for the leaf name. When the
    /// blob is absent, depth defaults to [`DEFAULT_FANOUT`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Reference`] if the ref does not exist, or
    /// [`Error::InvalidFanoutDepth`] if the `.fanout` entry is a tree or
    /// the blob contents are not a valid depth.
    ///
    /// [`metadata_default_ref`]: MetadataRepository::metadata_default_ref
    /// [`DEFAULT_FANOUT`]: crate::DEFAULT_FANOUT
    fn metadata_ref_fanout(&self, metadatas_ref: Option<&str>) -> Result<u8, Error>;

    fn metadata_delete(
        &self,
        id: gix::ObjectId,
        metadatas_ref: Option<&str>,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
    ) -> Result<(), Error>;

    /// Returns every valid [`Metadata`] entry reachable from `metadatas_ref`.
    ///
    /// When `metadatas_ref` is `None`, [`metadata_default_ref`] is used. The
    /// referenced tree is walked breadth-first; entries that match the fanout
    /// shape described on [`MetadataRepository`] are passed to
    /// [`Metadata::new`] for verification. Non-matching entries are skipped.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Reference`] if the ref does not exist,
    /// [`Error::InvalidFanoutDepth`] if a `.fanout` blob is present but
    /// malformed, or any error from [`Metadata::new`] on the first invalid
    /// leaf.
    ///
    /// # Examples
    ///
    /// Reading an empty fanout tree:
    ///
    /// ```
    /// use git_metadata::MetadataRepository;
    /// use gix::refs::transaction::PreviousValue;
    ///
    /// let dir = tempfile::tempdir().expect("tempdir");
    /// let repo = gix::init(dir.path()).expect("init repository");
    /// let empty = repo.write_object(gix::objs::Tree::empty())?.detach();
    /// repo.reference("refs/metadata/commits", empty, PreviousValue::Any, "init")?;
    ///
    /// let entries = repo.metadatas(None)?;
    /// assert!(entries.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`metadata_default_ref`]: MetadataRepository::metadata_default_ref
    fn metadatas(&self, metadatas_ref: Option<&str>) -> Result<Vec<Metadata>, Error>;

    fn find_metadata(
        &self,
        metadatas_ref: Option<&str>,
        id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error>;
}
