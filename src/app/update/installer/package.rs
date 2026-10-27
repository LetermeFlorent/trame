use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::super::UPDATED_FLAG;

const INSTALLED_AT: &str = "/usr/bin/trame";
const ARCHITECTURE: &str = "x86_64";
const DPKG_RECORD: &str = "/var/lib/dpkg/info/trame.list";
const RPM: &str = "/usr/bin/rpm";
const PKEXEC: &str = "/usr/bin/pkexec";
const SHELL: &str = "/bin/sh";
const DEB_TOOLS: [&[&str]; 2] = [
    &["/usr/bin/apt-get", "install", "-y"],
    &["/usr/bin/dpkg", "-i"],
];
const RPM_TOOLS: [&[&str]; 3] = [
    &["/usr/bin/dnf", "install", "-y"],
    &[
        "/usr/bin/zypper",
        "--non-interactive",
        "install",
        "--allow-unsigned-rpm",
    ],
    &[RPM, "-U"],
];
const CACHE_FOLDERS: [&str; 2] = ["trame", "update"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Package {
    Deb,
    Rpm,
}

impl Package {
    pub fn of(exe: &Path) -> Option<Self> {
        let usable =
            exe == Path::new(INSTALLED_AT) && env::consts::ARCH == ARCHITECTURE && exists(PKEXEC);
        usable
            .then(|| Self::pick(exists(DPKG_RECORD), exists(RPM)))
            .flatten()
    }

    fn pick(dpkg: bool, rpm: bool) -> Option<Self> {
        match (dpkg, rpm) {
            (true, _) => Some(Self::Deb),
            (false, true) => Some(Self::Rpm),
            (false, false) => None,
        }
    }

    pub fn asset(self, number: &str) -> String {
        match self {
            Self::Deb => format!("trame_{number}-1_amd64.deb"),
            Self::Rpm => format!("trame-{number}-1.x86_64.rpm"),
        }
    }

    pub fn install(self, file: &Path, exe: &Path) -> io::Result<()> {
        let tool = self
            .tool()
            .ok_or_else(|| io::Error::other("no package manager to install the update"))?;
        Command::new(SHELL)
            .args(["-c", &script()])
            .arg(exe)
            .args(tool)
            .arg(file)
            .stdin(Stdio::null())
            .spawn()
            .map(drop)
    }

    fn tool(self) -> Option<&'static [&'static str]> {
        let tools: &[&'static [&'static str]] = match self {
            Self::Deb => &DEB_TOOLS,
            Self::Rpm => &RPM_TOOLS,
        };
        tools
            .iter()
            .copied()
            .find(|tool| tool.first().is_some_and(|program| exists(program)))
    }
}

pub fn folder() -> Option<PathBuf> {
    let cache = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;
    Some(
        CACHE_FOLDERS
            .iter()
            .fold(cache, |path, part| path.join(part)),
    )
}

fn script() -> String {
    format!("{PKEXEC} \"$@\"; exec \"$0\" {UPDATED_FLAG}")
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARGO: &str = include_str!("../../../../Cargo.toml");

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
        assert!(script.starts_with(&format!("{PKEXEC} \"$@\";")));
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
}
