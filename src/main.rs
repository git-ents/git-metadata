mod cli;

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use clap::Parser;
use gix::objs::tree::EntryKind;

use cli::{Cli, Command};
use git_metadata::exe::{Executor, TreeEntry};

fn main() {
    let cli = Cli::parse();

    if cli.generate_man_page {
        let dir = cli.man_dir.clone().unwrap_or_else(default_man_dir);
        if let Err(e) = generate_man_page(dir, cli.force) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
        return;
    }

    if cli.command.is_none() {
        use clap::CommandFactory;
        let _ = Cli::command().print_help();
        eprintln!();
        process::exit(2);
    }

    if let Err(e) = run(&cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<()> {
    let executor = Executor::open(cli.repo.as_deref())?.with_ref(cli.r#ref.clone());
    let stdout = io::stdout();
    let mut out = stdout.lock();

    match cli.command.as_ref().unwrap() {
        Command::List { object, all } => {
            if *all {
                let mut first = true;
                for meta in executor.list_targets()? {
                    let entries = executor.ls_tree(meta.id())?;
                    if entries.is_empty() {
                        continue;
                    }
                    if !first {
                        writeln!(out)?;
                    }
                    first = false;
                    writeln!(out, "{}:", meta.id())?;
                    for entry in entries {
                        print_tree_entry(&mut out, &entry)?;
                    }
                }
            } else {
                let oid = executor.resolve_oid(object)?;
                for entry in executor.ls_tree(oid)? {
                    print_tree_entry(&mut out, &entry)?;
                }
            }
        }
        Command::Show { object } => {
            let oid = executor.resolve_oid(object)?;
            let entries = executor.ls_tree(oid)?;
            let tree = termtree::Tree::<String>::from(MetadataTree {
                label: oid.to_string(),
                entries,
            });
            writeln!(out, "{tree}")?;
        }
        Command::Add {
            path,
            object,
            message,
            file,
            link,
            link_ref,
            allow_empty,
            shard_level,
            force,
        } => {
            let oid = executor.resolve_oid(object)?;
            if let Some(rev) = link {
                let p = path
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("--path/-p is required with --link"))?;
                let link_oid = executor.resolve_oid(rev)?;
                let obj = executor.repo().find_object(link_oid)?.peel_tags_to_end()?;
                let kind = match obj.kind {
                    gix::object::Kind::Blob => EntryKind::Blob,
                    gix::object::Kind::Tree => EntryKind::Tree,
                    gix::object::Kind::Commit => EntryKind::Commit,
                    gix::object::Kind::Tag => unreachable!("peel_tags_to_end removes tags"),
                };
                executor.upsert(oid, p, kind, obj.id, *force, None, None, *shard_level)?;
            } else if let Some(ref_name) = link_ref {
                let p = path
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("--path/-p is required with --link-ref"))?;
                let blob_id = executor.repo().write_blob(ref_name.as_bytes())?.detach();
                executor.upsert(
                    oid,
                    p,
                    EntryKind::Blob,
                    blob_id,
                    *force,
                    None,
                    None,
                    *shard_level,
                )?;
            } else {
                let file_basename: Option<String> = file
                    .as_ref()
                    .and_then(|f| f.file_name())
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string());
                let p = path
                    .as_deref()
                    .or(file_basename.as_deref())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "--path/-p is required unless --file, --link, or --link-ref is given"
                        )
                    })?;
                let content = read_content(
                    message.as_deref(),
                    file.as_deref(),
                    executor.repo(),
                    object,
                    p,
                )?;
                if content.is_empty() && !allow_empty {
                    anyhow::bail!("refusing to add empty content; pass --allow-empty to override");
                }
                let blob_id = executor.repo().write_blob(&content)?.detach();
                executor.upsert(
                    oid,
                    p,
                    EntryKind::Blob,
                    blob_id,
                    *force,
                    None,
                    None,
                    *shard_level,
                )?;
            }
        }
        Command::Remove {
            patterns,
            object,
            keep,
        } => {
            let oid = executor.resolve_oid(object)?;
            let refs: Vec<&str> = patterns.iter().map(String::as_str).collect();
            if *keep {
                let to_remove = paths_not_matching(&executor, oid, &refs)?;
                let remove_refs: Vec<&str> = to_remove.iter().map(String::as_str).collect();
                executor.remove(oid, &remove_refs, None, None)?;
            } else {
                executor.remove(oid, &refs, None, None)?;
            }
        }
        Command::Copy { from, to, force } => {
            let from_oid = executor.resolve_oid(from)?;
            let to_oid = executor.resolve_oid(to)?;
            executor.copy(from_oid, to_oid, *force)?;
        }
        Command::Prune { dry_run, verbose } => {
            for oid in executor.prune(*dry_run)? {
                if *verbose {
                    writeln!(out, "{oid}")?;
                }
            }
        }
        Command::Edit {
            path,
            object,
            allow_empty,
        } => {
            let oid = executor.resolve_oid(object)?;
            let existing = executor.read_blob_at(oid, path)?;
            let edited = edit_blob_in_place(executor.repo(), object, path, &existing)?;
            if edited == existing {
                return Ok(());
            }
            if edited.is_empty() && !allow_empty {
                anyhow::bail!("refusing to save empty content; pass --allow-empty to override");
            }
            let blob_id = executor.repo().write_blob(&edited)?.detach();
            executor.upsert(oid, path, EntryKind::Blob, blob_id, true, None, None, 1)?;
        }
        Command::Merge { source, message } => {
            executor.merge(source, message.as_deref())?;
        }
        Command::GetRef => {
            writeln!(out, "{}", executor.metadatas_ref())?;
        }
    }

    Ok(())
}

/// Writes a single tree entry in `git ls-tree`-compatible format:
/// `<mode> <kind> <oid>\t<path>`.
fn print_tree_entry(out: &mut dyn Write, entry: &TreeEntry) -> Result<()> {
    writeln!(
        out,
        "{:06o} {} {}\t{}",
        entry.mode.value(),
        entry.mode.as_str(),
        entry.oid,
        entry.path,
    )?;
    Ok(())
}

/// Resolves the blob content for `add`, in precedence order: an
/// inline `--message`, then a `--file`, then an interactive editor
/// (when stdin and stderr are both TTYs), and finally piped stdin.
/// Errors if stdin is a TTY but stderr is not, since no editor can
/// usefully run in that case.
fn read_content(
    message: Option<&str>,
    file: Option<&Path>,
    repo: &gix::Repository,
    object: &str,
    path: &str,
) -> Result<Vec<u8>> {
    if let Some(msg) = message {
        return Ok(msg.as_bytes().to_vec());
    }
    if let Some(path) = file {
        return std::fs::read(path).with_context(|| format!("reading {path:?}"));
    }
    if atty_stdin() {
        use std::io::IsTerminal;
        if !io::stderr().is_terminal() {
            anyhow::bail!(
                "no content provided and stderr is not a terminal; pass --message/-m, --file/-F, or pipe content to stdin"
            );
        }
        return edit_in_editor(repo, object, path);
    }
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

/// Writes a `METADATA_EDITMSG` template under the git dir, launches
/// the resolved editor, and returns the saved content with comment
/// lines stripped. Honors git's editor precedence by shadowing
/// `VISUAL` with `GIT_EDITOR`/`core.editor` before delegating to the
/// [`edit`] crate.
fn edit_in_editor(repo: &gix::Repository, object: &str, path: &str) -> Result<Vec<u8>> {
    let edit_path = repo.git_dir().join("METADATA_EDITMSG");
    let template = format!(
        "# Please enter the metadata content for `{path}` on `{object}`.\n\
         # Lines starting with '#' will be ignored, and an empty message aborts the entry.\n\
         # Pass --allow-empty to allow empty content.\n"
    );
    std::fs::write(&edit_path, &template)
        .with_context(|| format!("writing edit template to {edit_path:?}"))?;

    if let Some(editor) = git_editor_override(repo) {
        // SAFETY: single-threaded CLI; no other thread reads the
        // environment concurrently. Mirrors git's precedence by
        // making GIT_EDITOR/core.editor visible to `edit`, which
        // reads VISUAL first.
        unsafe {
            std::env::set_var("VISUAL", &editor);
        }
    }
    edit::edit_file(&edit_path).context("launching editor")?;

    let raw = std::fs::read(&edit_path).with_context(|| format!("reading {edit_path:?}"))?;
    Ok(strip_comments(&raw))
}

/// Seeds `METADATA_EDITMSG` with `existing`, launches the editor, and
/// returns the resulting bytes verbatim. Unlike [`edit_in_editor`], no
/// comment template is prepended and `#`-prefixed lines are preserved —
/// the blob may legitimately contain them.
fn edit_blob_in_place(
    repo: &gix::Repository,
    object: &str,
    path: &str,
    existing: &[u8],
) -> Result<Vec<u8>> {
    let edit_path = repo.git_dir().join("METADATA_EDITMSG");
    std::fs::write(&edit_path, existing)
        .with_context(|| format!("writing edit buffer to {edit_path:?}"))?;

    if let Some(editor) = git_editor_override(repo) {
        // SAFETY: see `edit_in_editor`.
        unsafe {
            std::env::set_var("VISUAL", &editor);
        }
    }
    edit::edit_file(&edit_path)
        .with_context(|| format!("launching editor for {path:?} on {object}"))?;

    std::fs::read(&edit_path).with_context(|| format!("reading {edit_path:?}"))
}

/// Returns the editor command from `GIT_EDITOR` or `core.editor` (in
/// that order), or `None` if neither is set. The returned string may
/// contain shell-quoted arguments (e.g. `code --wait`).
fn git_editor_override(repo: &gix::Repository) -> Option<String> {
    if let Ok(v) = std::env::var("GIT_EDITOR")
        && !v.is_empty()
    {
        return Some(v);
    }
    let snapshot = repo.config_snapshot();
    if let Some(v) = snapshot.string("core.editor")
        && !v.is_empty()
    {
        return Some(v.to_string());
    }
    None
}

/// Drops every line whose first byte is `#`, preserving all other
/// bytes verbatim (including any trailing newline). Matches the
/// convention used by `git commit`'s editor template.
fn strip_comments(buf: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(buf.len());
    for line in buf.split_inclusive(|&b| b == b'\n') {
        if line.first().copied() == Some(b'#') {
            continue;
        }
        out.extend_from_slice(line);
    }
    out
}

/// Returns every metadata path on `target` that matches none of the
/// given globs. Used by `remove --keep` to invert the user's pattern
/// list into a set of paths to delete. Slashes in paths are treated
/// literally (no recursive `**` semantics).
fn paths_not_matching(
    executor: &Executor,
    target: gix::ObjectId,
    patterns: &[&str],
) -> Result<Vec<String>> {
    let compiled: Vec<gix::glob::Pattern> = patterns
        .iter()
        .map(|p| {
            gix::glob::parse(p.as_bytes())
                .ok_or_else(|| anyhow::anyhow!("invalid glob pattern: {p:?}"))
        })
        .collect::<Result<_>>()?;

    Ok(executor
        .ls_tree(target)?
        .into_iter()
        .filter(|entry| {
            !compiled.iter().any(|p| {
                p.matches(
                    gix::bstr::BStr::new(entry.path.as_bytes()),
                    gix::glob::wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
                )
            })
        })
        .map(|entry| entry.path)
        .collect())
}

/// Reports whether stdin is connected to a terminal.
fn atty_stdin() -> bool {
    use std::io::IsTerminal;
    io::stdin().is_terminal()
}

/// Resolves the default install location for the generated man page,
/// using `$XDG_DATA_HOME/man/man1` when set, then `$HOME/.local/share/
/// man/man1`, and finally the current working directory.
fn default_man_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("man/man1")
}

struct MetadataTree {
    label: String,
    entries: Vec<TreeEntry>,
}

impl From<MetadataTree> for termtree::Tree<String> {
    fn from(mt: MetadataTree) -> Self {
        let paths: Vec<&str> = mt.entries.iter().map(|e| e.path.as_str()).collect();
        build_subtree(mt.label, &paths)
    }
}

fn build_subtree(label: String, paths: &[&str]) -> termtree::Tree<String> {
    let mut tree = termtree::Tree::new(label);
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for path in paths {
        if let Some((head, tail)) = path.split_once('/') {
            groups.entry(head).or_default().push(tail);
        } else {
            groups.entry(path).or_default();
        }
    }
    for (key, children) in groups {
        if children.is_empty() {
            tree.push(termtree::Tree::new(key.to_owned()));
        } else {
            tree.push(build_subtree(key.to_owned(), &children));
        }
    }
    tree
}

/// Renders the clap-derived CLI into a `git-metadata.1` roff file
/// inside `output_dir`. Refuses to overwrite an existing file unless
/// `force` is true.
fn generate_man_page(output_dir: PathBuf, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    use clap::CommandFactory;

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }

    let path = output_dir.join("git-metadata.1");
    if path.exists() && !force {
        return Err(format!(
            "{} already exists; pass --force to overwrite",
            path.display()
        )
        .into());
    }
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    std::fs::write(&path, buf)?;
    println!("{}", path.display());
    Ok(())
}

/// Reports whether the given directory appears as a colon-separated
/// segment of `$MANPATH`. Reserved for a future post-install hint
/// when the man page is written outside the user's configured path.
#[allow(dead_code)]
fn manpath_covers(dir: &Path) -> bool {
    std::env::var("MANPATH")
        .map(|p| p.split(':').any(|seg| Path::new(seg) == dir))
        .unwrap_or(false)
}
