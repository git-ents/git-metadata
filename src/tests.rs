use super::*;
use git2::Repository;

fn init_repo() -> (tempfile::TempDir, Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();

    // Configure a dummy signature so commits work.
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "test").unwrap();
    config.set_str("user.email", "test@test").unwrap();

    (dir, repo)
}

fn make_tree(repo: &Repository) -> Oid {
    let blob = repo.blob(b"hello").unwrap();
    let mut builder = repo.treebuilder(None).unwrap();
    builder.insert("file.txt", blob, 0o100644).unwrap();
    builder.write().unwrap()
}

fn make_target(repo: &Repository) -> Oid {
    // Any OID works as a target key; use a blob for simplicity.
    repo.blob(b"target object").unwrap()
}

#[test]
fn set_and_get() {
    let (_dir, repo) = init_repo();
    let target = make_target(&repo);
    let tree_oid = make_tree(&repo);
    let opts = MetadataOptions::default();

    repo.metadata_set("refs/metadata/test", &target, &tree_oid, &opts)
        .unwrap();

    let got = repo.metadata_get("refs/metadata/test", &target).unwrap();
    assert_eq!(got, Some(tree_oid));
}

#[test]
fn get_missing_returns_none() {
    let (_dir, repo) = init_repo();
    let target = make_target(&repo);

    let got = repo.metadata_get("refs/metadata/test", &target).unwrap();
    assert_eq!(got, None);
}

#[test]
fn set_without_force_errors_on_duplicate() {
    let (_dir, repo) = init_repo();
    let target = make_target(&repo);
    let tree_oid = make_tree(&repo);
    let opts = MetadataOptions {
        force: false,
        ..Default::default()
    };

    repo.metadata_set("refs/metadata/test", &target, &tree_oid, &opts)
        .unwrap();

    let result = repo.metadata_set("refs/metadata/test", &target, &tree_oid, &opts);
    assert!(result.is_err());
}

#[test]
fn set_with_force_overwrites() {
    let (_dir, repo) = init_repo();
    let target = make_target(&repo);
    let tree1 = make_tree(&repo);

    let blob2 = repo.blob(b"other").unwrap();
    let mut b = repo.treebuilder(None).unwrap();
    b.insert("other.txt", blob2, 0o100644).unwrap();
    let tree2 = b.write().unwrap();

    let opts = MetadataOptions {
        force: true,
        ..Default::default()
    };

    repo.metadata_set("refs/metadata/test", &target, &tree1, &opts)
        .unwrap();
    repo.metadata_set("refs/metadata/test", &target, &tree2, &opts)
        .unwrap();

    let got = repo.metadata_get("refs/metadata/test", &target).unwrap();
    assert_eq!(got, Some(tree2));
}

#[test]
fn remove_existing() {
    let (_dir, repo) = init_repo();
    let target = make_target(&repo);
    let tree_oid = make_tree(&repo);
    let opts = MetadataOptions::default();

    repo.metadata_set("refs/metadata/test", &target, &tree_oid, &opts)
        .unwrap();

    let removed = repo
        .metadata_remove("refs/metadata/test", &target, &opts)
        .unwrap();
    assert!(removed);

    let got = repo.metadata_get("refs/metadata/test", &target).unwrap();
    assert_eq!(got, None);
}

#[test]
fn remove_nonexistent() {
    let (_dir, repo) = init_repo();
    let target = make_target(&repo);
    let opts = MetadataOptions::default();

    let removed = repo
        .metadata_remove("refs/metadata/test", &target, &opts)
        .unwrap();
    assert!(!removed);
}

#[test]
fn list_entries() {
    let (_dir, repo) = init_repo();
    let t1 = repo.blob(b"a").unwrap();
    let t2 = repo.blob(b"b").unwrap();
    let tree1 = make_tree(&repo);

    let blob2 = repo.blob(b"other").unwrap();
    let mut b = repo.treebuilder(None).unwrap();
    b.insert("x.txt", blob2, 0o100644).unwrap();
    let tree2 = b.write().unwrap();

    let opts = MetadataOptions::default();

    repo.metadata_set("refs/metadata/test", &t1, &tree1, &opts)
        .unwrap();
    repo.metadata_set("refs/metadata/test", &t2, &tree2, &opts)
        .unwrap();

    let entries = repo.metadata_list("refs/metadata/test").unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.contains(&(t1, tree1)));
    assert!(entries.contains(&(t2, tree2)));
}
