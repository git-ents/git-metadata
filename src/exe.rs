use std::path::Path;

pub fn open_repo(path: Option<&Path>) {
    todo!()
}

pub fn resolve_oid(repo: &(), rev: &str) {
    todo!()
}

pub fn list(repo: &(), ref_name: &str) {
    todo!()
}

pub fn show(repo: &(), ref_name: &str, target: &()) {
    todo!()
}

pub fn add(repo: &(), ref_name: &str, target: &(), path: &str, content: Option<&[u8]>, opts: &()) {
    todo!()
}

pub fn remove_paths(repo: &(), ref_name: &str, target: &(), patterns: &[&str], keep: bool) {
    todo!()
}

pub fn copy(repo: &(), ref_name: &str, from: &(), to: &(), opts: &()) {
    todo!()
}

pub fn prune(repo: &(), ref_name: &str, dry_run: bool) {
    todo!()
}

pub fn get_ref(repo: &(), ref_name: &str) {
    todo!()
}

pub fn link(
    repo: &(),
    ref_name: &str,
    a: &str,
    b: &str,
    forward: &str,
    reverse: &str,
    meta: Option<&[u8]>,
) {
    todo!()
}

pub fn unlink(repo: &(), ref_name: &str, a: &str, b: &str, forward: &str, reverse: &str) {
    todo!()
}

pub fn linked(repo: &(), ref_name: &str, key: &str, relation: Option<&str>) {
    todo!()
}
