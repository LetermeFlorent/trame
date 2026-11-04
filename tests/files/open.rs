use std::fs;
use std::path::{Path, PathBuf};

use trame::files::{LinkTarget, Opening, plan_opening};

use super::Sandbox;

fn sandbox(name: &str) -> Sandbox {
    let sandbox = Sandbox::new(&format!("open-{name}"));
    for file in [
        "notes.txt",
        "run.bat",
        "Setup.EXE",
        "tool.sh",
        "sub/inner.md",
    ] {
        let path = sandbox.root.join(file);
        fs::create_dir_all(path.parent().expect("a parent")).expect("the folder is made");
        fs::write(path, b"x").expect("the file is written");
    }
    sandbox
}

fn same(left: &Path, right: &Path) -> bool {
    fs::canonicalize(left).ok() == fs::canonicalize(right).ok()
}

fn opened(target: &LinkTarget, cwd: Option<&Path>) -> Option<Opening> {
    plan_opening(target, cwd, None)
}

fn file_at(opening: Option<Opening>) -> PathBuf {
    match opening {
        Some(Opening::File(path)) => path,
        other => panic!("not a file: {other:?}"),
    }
}

fn folder_at(opening: Option<Opening>) -> PathBuf {
    match opening {
        Some(Opening::Folder(path)) => path,
        other => panic!("not a folder: {other:?}"),
    }
}

fn file_url(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    format!("file:///{}", text.trim_start_matches('/'))
}

#[test]
fn only_http_and_https_addresses_go_to_the_browser() {
    for url in ["https://example.com/a", "http://x.org", "HTTPS://A.EXAMPLE"] {
        let wanted = Some(Opening::Browser(url.to_owned()));
        assert_eq!(opened(&LinkTarget::Web(url.to_owned()), None), wanted);
        assert_eq!(opened(&LinkTarget::Uri(url.to_owned()), None), wanted);
    }
    for uri in [
        "javascript:alert(1)",
        "ftp://example.com/file",
        "ms-settings:display",
        "vscode://file/c:/x",
        "file://elsewhere.example/etc/hosts",
        "https://",
        "https:///path",
        "https://a.example/\"x",
        "https://a.example/ x",
    ] {
        assert_eq!(
            opened(&LinkTarget::Uri(uri.to_owned()), None),
            None,
            "{uri}"
        );
    }
}

#[test]
fn a_file_opens_only_when_it_exists() {
    let sandbox = sandbox("exists");
    let root = sandbox.root.as_path();
    let absolute = root.join("notes.txt").to_string_lossy().into_owned();
    let found = file_at(opened(&LinkTarget::Path(absolute), None));
    assert!(same(&found, &root.join("notes.txt")));
    let missing = root.join("gone.txt").to_string_lossy().into_owned();
    assert_eq!(opened(&LinkTarget::Path(missing), Some(root)), None);
}

#[test]
fn relative_paths_start_from_the_folder_of_the_tab() {
    let sandbox = sandbox("relative");
    let root = sandbox.root.as_path();
    let inner = LinkTarget::Path("sub/inner.md".to_owned());
    assert!(same(
        &file_at(opened(&inner, Some(root))),
        &root.join("sub/inner.md")
    ));
    let dotted = LinkTarget::Path("./sub/../notes.txt".to_owned());
    assert!(same(
        &file_at(opened(&dotted, Some(root))),
        &root.join("notes.txt")
    ));
    assert_eq!(opened(&inner, None), None);
    assert_eq!(opened(&inner, Some(Path::new("relative/folder"))), None);
    let home = LinkTarget::Path("~/notes.txt".to_owned());
    let found = file_at(plan_opening(&home, None, Some(root)));
    assert!(same(&found, &root.join("notes.txt")));
}

#[test]
fn programs_and_scripts_open_their_folder_instead() {
    let sandbox = sandbox("programs");
    let root = sandbox.root.as_path();
    for name in ["run.bat", "Setup.EXE", "tool.sh"] {
        let folder = folder_at(opened(&LinkTarget::Path(name.to_owned()), Some(root)));
        assert!(same(&folder, root), "{name}");
    }
    let url = file_url(&root.join("run.bat"));
    assert!(same(&folder_at(opened(&LinkTarget::Uri(url), None)), root));
    let sub = LinkTarget::Path("sub".to_owned());
    assert!(same(
        &folder_at(opened(&sub, Some(root))),
        &root.join("sub")
    ));
}

#[test]
fn a_local_file_address_opens_the_file() {
    let sandbox = sandbox("address");
    let notes = sandbox.root.join("notes.txt");
    let found = file_at(opened(&LinkTarget::Uri(file_url(&notes)), None));
    assert!(same(&found, &notes));
}

#[cfg(unix)]
#[test]
fn a_file_with_an_execute_bit_opens_its_folder() {
    use std::os::unix::fs::PermissionsExt;

    let sandbox = sandbox("bit");
    let script = sandbox.root.join("launcher");
    fs::write(&script, b"x").expect("the file is written");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("the mode is set");
    let target = LinkTarget::Path(script.to_string_lossy().into_owned());
    assert!(same(&folder_at(opened(&target, None)), &sandbox.root));
}
