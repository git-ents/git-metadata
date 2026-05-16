use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "git metadata", bin_name = "git metadata")]
#[command(
    author,
    version,
    about = "Manage Git object metadata stored in a fanout ref tree.",
    long_about = None
)]
pub struct Cli {
    /// Path to the git repository. Defaults to the current directory.
    #[arg(short = 'C', long, global = true)]
    pub repo: Option<PathBuf>,

    /// The ref under which metadata is stored.
    #[arg(long, global = true, default_value = "refs/metadata/objects")]
    pub r#ref: String,

    /// Write the man page and exit. Installs to $XDG_DATA_HOME/man/man1 unless --man-dir is given.
    #[arg(long)]
    pub generate_man_page: bool,

    /// Directory to write the man page into. Requires --generate-man-page.
    #[arg(long, value_name = "DIR", requires = "generate_man_page")]
    pub man_dir: Option<PathBuf>,

    /// Overwrite existing entries / files without error. Used with --generate-man-page to
    /// overwrite an existing file; for subcommands, pass --force after the subcommand name.
    #[arg(short, long)]
    pub force: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// List the metadata tree entries for an object.
    List {
        /// The target object (OID or revision). Defaults to HEAD.
        #[arg(default_value = "HEAD")]
        object: String,
    },

    /// Show the metadata tree for an object as a tree.
    Show {
        /// The target object (OID or revision). Defaults to HEAD.
        #[arg(default_value = "HEAD")]
        object: String,
    },

    /// Add an entry to an object's metadata tree.
    Add {
        /// Path within the metadata tree (e.g. `labels/bug`).
        /// Required unless --file, --link, or --link-ref is given; --file defaults to the file's basename.
        #[arg(short = 'p', long)]
        path: Option<String>,

        /// The target object (OID or revision). Defaults to HEAD.
        #[arg(default_value = "HEAD")]
        object: String,

        /// Content to store in the blob. Reads from stdin when omitted.
        #[arg(short, long, conflicts_with_all = ["file", "link", "link_ref"])]
        message: Option<String>,

        /// Read content from a file on disk. Path defaults to the file's basename.
        #[arg(short = 'F', long, conflicts_with_all = ["message", "link", "link_ref"])]
        file: Option<PathBuf>,

        /// Link an existing Git object (any type) at the path. Resolved to an OID at write time.
        #[arg(long, conflicts_with_all = ["message", "file", "link_ref"])]
        link: Option<String>,

        /// Store a ref name as a blob at the path (symbolic pointer, not resolved).
        #[arg(long = "link-ref", conflicts_with_all = ["message", "file", "link"])]
        link_ref: Option<String>,

        /// Allow adding an entry with empty content.
        #[arg(long)]
        allow_empty: bool,

        /// Fanout depth (number of 2-hex-char directory segments, max 19).
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(0..=19))]
        shard_level: u8,

        /// Overwrite an existing entry at the same path without error.
        #[arg(short, long)]
        force: bool,
    },

    /// Remove path entries from an object's metadata tree.
    Remove {
        /// Glob patterns for entries to remove (or keep with `--keep`).
        patterns: Vec<String>,

        /// The target object (OID or revision). Defaults to HEAD.
        #[arg(default_value = "HEAD")]
        object: String,

        /// Invert: keep only entries matching the patterns.
        #[arg(long)]
        keep: bool,
    },

    /// Copy metadata from one object to another.
    ///
    /// With --force, replaces the destination's entire metadata tree wholesale;
    /// any entries on the destination not present in the source are dropped.
    Copy {
        /// The source object (OID or revision).
        from: String,

        /// The destination object (OID or revision).
        to: String,

        /// Replace the destination's metadata tree; drops entries not in the source.
        #[arg(short, long)]
        force: bool,
    },

    /// Remove metadata for objects that no longer exist.
    Prune {
        /// Only report what would be pruned; do not actually remove.
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Print each pruned object.
        #[arg(short, long)]
        verbose: bool,
    },

    /// Print the metadata ref name.
    GetRef,
}
