use std::fs::{self, Permissions};
use std::os::unix::fs::{PermissionsExt, symlink};

use trame::files::write_atomic;

use super::Sandbox;

#[test]
fn a_rewritten_file_keeps_its_mode() {
    let sandbox = Sandbox::new("atomic-mode");
    let path = sandbox.root.join("config");
    fs::write(&path, "old").expect("written");
    fs::set_permissions(&path, Permissions::from_mode(0o600)).expect("narrowed");
    write_atomic(&path, b"new").expect("rewritten");
    let mode = fs::metadata(&path).expect("read").permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn a_linked_file_is_rewritten_behind_its_link() {
    let sandbox = Sandbox::new("atomic-link");
    let real = sandbox.root.join("dotfiles-config");
    let link = sandbox.root.join("config");
    fs::write(&real, "old").expect("written");
    symlink(&real, &link).expect("linked");
    write_atomic(&link, b"new").expect("rewritten");
    let kind = fs::symlink_metadata(&link).expect("read").file_type();
    assert!(kind.is_symlink());
    assert_eq!(fs::read(&real).expect("read"), b"new");
}
