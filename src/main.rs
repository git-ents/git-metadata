mod cli;

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use gix::objs::tree::EntryKind;

use cli::{Cli, Command};
use git_metadata::exe::{Executor, TreeEntry};

fn main() {
    if let Some(dir) = parse_generate_man_flag() {
        if let Err(e) = generate_man_page(dir) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<()> {
    let executor = Executor::open(cli.repo.as_deref())?.with_ref(cli.r#ref.clone());
    let stdout = io::stdout();
    let mut out = stdout.lock();

    match &cli.command {
        Command::List => {
            for m in executor.list_targets()? {
                writeln!(out, "{} {}", m.id(), m.data())?;
            }
        }
        Command::Show { object } => {
            let oid = executor.resolve_oid(object)?;
            for entry in executor.ls_tree(oid)? {
                print_tree_entry(&mut out, &entry)?;
            }
        }
        Command::Add {
            path,
            object,
            message,
            file,
            link,
            link_ref,
            force,
            allow_empty,
            shard_level: _,
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
                executor.upsert(oid, p, kind, obj.id, *force, None, None)?;
            } else if let Some(ref_name) = link_ref {
                let p = path
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("--path/-p is required with --link-ref"))?;
                let blob_id = executor.repo().write_blob(ref_name.as_bytes())?.detach();
                executor.upsert(oid, p, EntryKind::Blob, blob_id, *force, None, None)?;
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
                executor.upsert(oid, p, EntryKind::Blob, blob_id, *force, None, None)?;
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
        Command::Copy {
            from,
            to,
            force,
            shard_level: _,
        } => {
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
        Command::GetRef => {
            writeln!(out, "{}", executor.metadatas_ref())?;
        }
    }

    Ok(())
}

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

fn edit_in_editor(repo: &gix::Repository, object: &str, path: &str) -> Result<Vec<u8>> {
    let edit_path = repo.git_dir().join("METADATA_EDITMSG");
    let template = format!(
        "\n\
         # Please enter the metadata content for `{path}` on `{object}`.\n\
         # Lines starting with '#' will be ignored, and an empty message aborts the entry,\n# if the `--allow-empty` flag was not provided.\n"
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

fn atty_stdin() -> bool {
    use std::io::IsTerminal;
    io::stdin().is_terminal()
}

fn parse_generate_man_flag() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--generate-man-page" {
            return Some(
                args.next()
                    .map(PathBuf::from)
                    .unwrap_or_else(default_man_dir),
            );
        }
        if let Some(dir) = arg.strip_prefix("--generate-man-page=") {
            return Some(PathBuf::from(dir));
        }
    }
    None
}

fn default_man_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("man/man1")
}

fn generate_man_page(output_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use clap::CommandFactory;
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    let path = output_dir.join("git-metadata.1");
    std::fs::write(&path, buf)?;
    Ok(())
}

#[allow(dead_code)]
fn manpath_covers(dir: &Path) -> bool {
    std::env::var("MANPATH")
        .map(|p| p.split(':').any(|seg| Path::new(seg) == dir))
        .unwrap_or(false)
}
