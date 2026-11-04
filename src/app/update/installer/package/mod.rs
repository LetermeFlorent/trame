#[cfg(test)]
mod tests;

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
const ROOT_NAME: &str = "trame-update";
const ROOT_SCRIPT: &str = r#"expected=$1 file=$2
shift 2
folder=$(mktemp -d) || exit
copy=$folder/${file##*/}
cp -- "$file" "$copy" && test "$(sha512sum < "$copy" | cut -d ' ' -f 1)" = "$expected" && "$@" "$copy"
status=$?
rm -rf -- "$folder"
exit "$status""#;

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

    pub fn install(self, file: &Path, expected: &str, exe: &Path) -> io::Result<()> {
        let tool = self
            .tool()
            .ok_or_else(|| io::Error::other("no package manager to install the update"))?;
        Command::new(SHELL)
            .args(["-c", &script()])
            .arg(exe)
            .args([ROOT_SCRIPT, ROOT_NAME, expected])
            .arg(file)
            .args(tool)
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
    format!("{PKEXEC} {SHELL} -c \"$@\"; exec \"$0\" {UPDATED_FLAG}")
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}
