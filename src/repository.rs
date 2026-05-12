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
    /// Attaches `metadata` to `oid` under `metadatas_ref` and returns the new
    /// commit id at the tip of the ref.
    ///
    /// When `metadatas_ref` is `None`, [`metadata_default_ref`] is used. The
    /// `metadata` argument must be a tree object; it is inserted into the
    /// fanout tree at the path derived from `oid` and the depth described on
    /// [`MetadataRepository`]. If the ref does not yet exist, an empty tree is
    /// created and depth defaults to [`DEFAULT_FANOUT`]. If the ref points at
    /// a bare tree (a bootstrap shape), it is migrated to a commit on first
    /// write.
    ///
    /// The `metadata` tree is stored at the fanout path as a single entry; its
    /// contents are **not** merged with any existing tree at that path. When
    /// `force` is `false`, the call fails with [`Error::AlreadyExists`] if a
    /// leaf is already present. When `true`, the existing leaf's oid is
    /// replaced wholesale by `metadata` — entries present in the prior tree
    /// but absent from `metadata` are dropped.
    ///
    /// The ref update is guarded with a compare-and-swap against the snapshot
    /// taken at the start of the call, so a concurrent writer's commit will
    /// cause this call to fail rather than be silently clobbered.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedHashKind`] if `oid`'s hash kind differs
    /// from the repository's, [`Error::NotFound`] if `metadata` is not in the
    /// object database, [`Error::InvalidType`] if `metadata` is not a tree,
    /// [`Error::InvalidRootType`] if the ref points at something other than a
    /// tree or commit, [`Error::AlreadyExists`] if a leaf already exists and
    /// `force` is `false`, [`Error::FanoutPathConflict`] if the fanout path
    /// collides with an existing non-tree entry, or [`Error::Gix`] for any
    /// underlying `gix` failure (including a failed CAS on the ref update).
    ///
    /// [`metadata_default_ref`]: MetadataRepository::metadata_default_ref
    /// [`DEFAULT_FANOUT`]: crate::DEFAULT_FANOUT
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
    /// Returns [`Error::Gix`] if the ref does not exist,
    /// [`Error::InvalidFanoutType`] if the `.fanout` entry is not a blob,
    /// or [`Error::InvalidFanoutDepth`] if the blob contents are not a
    /// valid depth.
    ///
    /// [`metadata_default_ref`]: MetadataRepository::metadata_default_ref
    /// [`DEFAULT_FANOUT`]: crate::DEFAULT_FANOUT
    fn metadata_ref_fanout(&self, metadatas_ref: Option<&str>) -> Result<u8, Error>;

    /// Removes the metadata leaf attached to `id` under `metadatas_ref`.
    ///
    /// When `metadatas_ref` is `None`, [`metadata_default_ref`] is used. The
    /// fanout tree is rewritten with the leaf at `id`'s fanout path removed,
    /// and a new commit authored by `author` and committed by `committer` is
    /// written to the tip of the ref.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if no leaf exists for `id`, or
    /// [`Error::Gix`] for any underlying `gix` failure.
    ///
    /// [`metadata_default_ref`]: MetadataRepository::metadata_default_ref
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
    /// Returns [`Error::Gix`] if the ref does not exist,
    /// [`Error::InvalidFanoutDepth`] if a `.fanout` blob is present but
    /// malformed, or any error from [`Metadata::new`] on the first leaf whose
    /// referenced objects fail verification.
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

    /// Returns the id of the metadata tree attached to `id` under
    /// `metadatas_ref`.
    ///
    /// When `metadatas_ref` is `None`, [`metadata_default_ref`] is used. The
    /// fanout tree is walked using the depth described on
    /// [`MetadataRepository`] to locate the leaf whose path matches `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if no leaf exists for `id`, or
    /// [`Error::Gix`] for any underlying `gix` failure.
    ///
    /// [`metadata_default_ref`]: MetadataRepository::metadata_default_ref
    fn find_metadata(
        &self,
        metadatas_ref: Option<&str>,
        id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error>;
}
