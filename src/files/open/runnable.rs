use std::fs::Metadata;
use std::path::Path;

const RUNNABLE: &str = "exe com bat cmd ps1 psm1 psd1 ps1xml psc1 msi msp msix msixbundle appx
    appxbundle appinstaller lnk url pif scr vbs vbe js jse wsf wsh ws hta cpl reg
    inf msc msh gadget chm jar appref-ms application settingcontent-ms library-ms
    search-ms diagcab scf sct py pyw pl rb sh bash zsh csh ksh fish command tool
    terminal workflow scpt applescript webloc inetloc fileloc pkg mpkg app prefpane
    desktop run bin appimage";

pub(super) fn is_runnable(path: &Path, metadata: &Metadata) -> bool {
    let named = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.trim_end_matches(['.', ' ']).to_ascii_lowercase())
        .is_some_and(|extension| RUNNABLE.split_whitespace().any(|known| known == extension));
    named || (metadata.is_file() && executable(metadata))
}

#[cfg(unix)]
fn executable(metadata: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn executable(_: &Metadata) -> bool {
    false
}
