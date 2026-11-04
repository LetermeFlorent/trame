use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellCommand {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
}

#[cfg(windows)]
const POWERSHELL_PROMPT_HOOK: &str = r"$global:__trame_prompt = $function:prompt; function global:prompt { $l = $executionContext.SessionState.Path.CurrentLocation; $o = ''; if ($l.Provider.Name -eq 'FileSystem') { $o = [char]27 + ']9;9;' + [char]34 + $l.ProviderPath + [char]34 + [char]27 + '\' }; $o + (& $global:__trame_prompt) }";

#[cfg(windows)]
const COMMAND_PROMPT: &str = r"$e]9;9;$P$e\$P$G";

#[cfg(windows)]
pub fn default_shell() -> ShellCommand {
    let path = std::env::var_os("PATH").unwrap_or_default();
    if let Some(pwsh) = find_in_path(&path, "pwsh.exe") {
        return powershell(pwsh);
    }
    let system_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let windows_powershell = system_root.join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    if windows_powershell.is_file() {
        return powershell(windows_powershell);
    }
    command_prompt(system_root)
}

#[cfg(windows)]
fn powershell(program: PathBuf) -> ShellCommand {
    let args = ["-NoLogo", "-NoExit", "-Command", POWERSHELL_PROMPT_HOOK];
    ShellCommand {
        program,
        args: args.into_iter().map(OsString::from).collect(),
        env: Vec::new(),
    }
}

#[cfg(windows)]
fn command_prompt(system_root: PathBuf) -> ShellCommand {
    let program = std::env::var_os("ComSpec")
        .map(PathBuf::from)
        .unwrap_or_else(|| system_root.join(r"System32\cmd.exe"));
    ShellCommand {
        program,
        args: Vec::new(),
        env: vec![(OsString::from("PROMPT"), OsString::from(COMMAND_PROMPT))],
    }
}

#[cfg(windows)]
pub fn find_in_path(path: &std::ffi::OsStr, name: &str) -> Option<PathBuf> {
    std::env::split_paths(path)
        .filter(|directory| directory.is_absolute())
        .map(|directory| directory.join(name))
        .find(|candidate| {
            std::fs::symlink_metadata(candidate).is_ok_and(|metadata| !metadata.is_dir())
        })
}

#[cfg(unix)]
pub fn default_shell() -> ShellCommand {
    let program = std::env::var_os("SHELL")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("/bin/sh"));
    let args = if cfg!(target_os = "macos") {
        vec![OsString::from("-l")]
    } else {
        Vec::new()
    };
    ShellCommand {
        program,
        args,
        env: Vec::new(),
    }
}
