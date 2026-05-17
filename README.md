# 📋 `git-metadata`

*Porcelain for adding metadata to any object without rewriting history.*

> [!NOTE]
> This project is in need of testers!
> Once enough testers have provided feedback, and the software's stability is demonstrated for daily usage in the wild, a major release will be published.
> Until then, please expect beta software quality for all pre-major (`<=v1.0`) releases.

## Overview

To support a more expansive usage of the Git object database — as is the goal for other projects within the [`git-ents`](https://github.com/git-ents) organization — new tooling is needed.
This project provides a command that allows users to associate arbitrary data to any object in Git's store.
The `metadata` command follows `notes` semantics.

[Notes] are a tragically underutilized feature of Git.
For more information about `git notes` entries, Tyler Cipriani's [blog post] is an excellent introduction, and some highly-motivating examples.
One such example is Google's open-source [`git-appraise`] project, which stores code review metadata as structured entries in a note blob.
While functional, that design highlights a limitation of notes: structured data, or data that does not map cleanly onto UTF-8 text, is difficult to represent in a blob format.
The `git-metadata` project provides a structured alternative to the notes-blob design using Git trees objects.
Just like notes, metadata added to an object does not alter the object's history.

> [!TIP]
> Unlike notes, `metadata` is not added to `git log`.

[Notes]: https://git-scm.com/docs/git-notes
[blog post]: https://tylercipriani.com/blog/2022/11/19/git-notes-gits-coolest-most-unloved-feature/
[`git-appraise`]: https://github.com/google/git-appraise

## Usage

Metadata entries are paths (with optional blob content) stored in a Git tree object, associated with any target object (blob, tree, or commit) via a fanout ref.
The command follows `git notes` semantics: `list`, `show`, `add`, `remove`, `copy`, `prune`, and `get-ref`.
See the example usage below for inspiration for how this command could be useful.

```bash
# Attach benchmark results to the current commit.
git metadata add --path bench/hyperfine.json --file results.json

# Attach logs that show a bug found on a previously released version.
git metadata add --path incident/$(date +%Y%m%dT%H%M%S).log --file incident.log v0.3.0-rc2

# Copy all logs to a newer commit, where the bug is still present.
git metadata copy v0.3.0-rc2 v0.3.0-rc3

# List all metadata entries on the current commit.
git metadata list

# Show all metadata entries structured as an in-terminal tree.
git metadata show
```

## Installation

### CLI

The `git-metadata` plumbing command can be installed with `cargo install`.

```shell
cargo install --locked git-metadata
```

If `~/.cargo/bin` is on your `PATH`, you can invoke the command with `git`.

```shell
git metadata -h
```

### Library

The `git-metadata` library can be added to your Rust project via `cargo add`.

```shell
cargo add git-metadata
```

## Code of Conduct

Please refer to the in-source [code of conduct](/CONDUCT.md).
