#![allow(dead_code)]

use std::path::Path;

pub fn open_repo(_path: Option<&Path>) {
    todo!()
}

pub fn resolve_oid(_repo: &(), _rev: &str) {
    todo!()
}

pub fn list(_repo: &(), _ref_name: &str) {
    todo!()
}

pub fn show(_repo: &(), _ref_name: &str, _target: &()) {
    todo!()
}

pub fn add(
    _repo: &(),
    _ref_name: &str,
    _target: &(),
    _path: &str,
    _content: Option<&[u8]>,
    _opts: &(),
) {
    todo!()
}

pub fn remove_paths(_repo: &(), _ref_name: &str, _target: &(), _patterns: &[&str], _keep: bool) {
    todo!()
}

pub fn copy(_repo: &(), _ref_name: &str, _from: &(), _to: &(), _opts: &()) {
    todo!()
}

pub fn prune(_repo: &(), _ref_name: &str, _dry_run: bool) {
    todo!()
}

pub fn get_ref(_repo: &(), _ref_name: &str) {
    todo!()
}

pub fn link(
    _repo: &(),
    _ref_name: &str,
    _a: &str,
    _b: &str,
    _forward: &str,
    _reverse: &str,
    _meta: Option<&[u8]>,
) {
    todo!()
}

pub fn unlink(_repo: &(), _ref_name: &str, _a: &str, _b: &str, _forward: &str, _reverse: &str) {
    todo!()
}

pub fn linked(_repo: &(), _ref_name: &str, _key: &str, _relation: Option<&str>) {
    todo!()
}
