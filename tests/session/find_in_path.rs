use std::path::Path;

#[test]
fn a_relative_path_entry_is_skipped() {
    use std::fs;
    use trame::session::pty::find_in_path;
    let program = "pwsh.exe";
    let tag = format!("trame-path-{}", std::process::id());
    let relative = Path::new("target").join(&tag);
    let absolute = std::env::temp_dir().join(&tag);
    for folder in [&relative, &absolute] {
        fs::create_dir_all(folder).expect("the folder");
        fs::write(folder.join(program), b"").expect("the program");
    }
    let path = std::env::join_paths([&relative, &absolute]).expect("a PATH value");
    let found = find_in_path(&path, program);
    for folder in [&relative, &absolute] {
        let _ = fs::remove_dir_all(folder);
    }
    assert_eq!(found, Some(absolute.join(program)));
}
