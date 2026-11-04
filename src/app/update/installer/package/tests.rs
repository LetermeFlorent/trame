use super::*;

const CARGO: &str = include_str!("../../../../../Cargo.toml");

#[test]
fn a_debian_system_gets_the_deb_even_with_rpm_around() {
    assert_eq!(Package::pick(true, true), Some(Package::Deb));
    assert_eq!(Package::pick(false, true), Some(Package::Rpm));
    assert_eq!(Package::pick(false, false), None);
}

#[test]
fn the_downloads_are_the_packages_the_release_publishes() {
    assert_eq!(Package::Deb.asset("0.4.0"), "trame_0.4.0-1_amd64.deb");
    assert_eq!(Package::Rpm.asset("0.4.0"), "trame-0.4.0-1.x86_64.rpm");
    assert!(CARGO.contains("\"usr/bin/\""));
    assert!(CARGO.contains("dest = \"/usr/bin/trame\""));
}

#[test]
fn the_password_prompt_installs_then_trame_opens_again_either_way() {
    let script = script();
    assert!(script.starts_with(&format!("{PKEXEC} {SHELL} -c \"$@\";")));
    assert!(script.ends_with(&format!("exec \"$0\" {UPDATED_FLAG}")));
}

#[test]
fn a_copy_outside_the_package_place_is_left_to_its_owner() {
    assert_eq!(Package::of(Path::new("/home/me/bin/trame")), None);
}

#[test]
fn downloads_wait_in_the_user_cache() {
    if let Some(folder) = folder() {
        assert!(folder.ends_with(Path::new("trame").join("update")));
    }
}

#[cfg(target_os = "linux")]
mod root_side {
    use std::fs;
    use std::process::ExitStatus;

    use crate::app::update::replace::signed;

    use super::*;

    #[test]
    fn the_password_prompt_installs_a_checked_copy_and_nothing_else() {
        let folder = std::env::temp_dir().join(format!("trame-package-{}", std::process::id()));
        fs::create_dir_all(&folder).expect("scratch folder");
        let package = folder.join("trame_9.9.9-1_amd64.deb");
        fs::write(&package, b"package").expect("a package");
        let installed = folder.join("installed");
        let record = "cp \"$1\" \"$0\" && printf %s \"$1\" > \"$0.from\"";
        let tool = ["/bin/sh", "-c", record, installed.to_str().expect("a path")];
        let expected = signed::digest(&b"package"[..]).expect("a digest");
        let refused = as_root(&"0".repeat(expected.len()), &package, &tool);
        let missed = installed.exists();
        let accepted = as_root(&expected, &package, &tool);
        let copied = fs::read(&installed).ok();
        let from = fs::read_to_string(folder.join("installed.from")).unwrap_or_default();
        fs::remove_dir_all(&folder).expect("cleanup");
        assert!(!refused.success() && !missed);
        assert!(accepted.success());
        assert_eq!(copied.as_deref(), Some(&b"package"[..]));
        assert!(Path::new(&from) != package && !Path::new(&from).exists());
    }

    fn as_root(expected: &str, package: &Path, tool: &[&str]) -> ExitStatus {
        Command::new(SHELL)
            .args(["-c", ROOT_SCRIPT, ROOT_NAME, expected])
            .arg(package)
            .args(tool)
            .status()
            .expect("the shell runs")
    }
}
