#[cfg(not(windows))]
use std::io;

#[cfg(windows)]
pub use registry::{menu_installed, set_menu};

#[cfg(not(windows))]
pub fn menu_installed() -> bool {
    false
}

#[cfg(not(windows))]
pub fn set_menu(_on: bool, _label: &str) -> io::Result<()> {
    Ok(())
}

#[cfg(windows)]
mod registry {
    use std::ffi::OsStr;
    use std::io;
    use std::ptr;

    use trame::utf16::wide;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR};
    use windows_sys::Win32::System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteTreeW, RegGetValueW, RegSetKeyValueW,
    };

    const ENTRIES: [&str; 2] = [
        r"Software\Classes\Directory\shell\trame",
        r"Software\Classes\Directory\Background\shell\trame",
    ];

    pub fn menu_installed() -> bool {
        let key = wide(OsStr::new(&format!(r"{}\command", ENTRIES[1])));
        let found = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                ptr::null(),
                RRF_RT_REG_SZ,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        found == ERROR_SUCCESS
    }

    pub fn set_menu(on: bool, label: &str) -> io::Result<()> {
        let exe = std::env::current_exe()?;
        let exe = exe.to_string_lossy();
        for entry in ENTRIES {
            if on {
                write(entry, None, label)?;
                write(entry, Some("Icon"), &format!("\"{exe}\",0"))?;
                write(
                    &format!(r"{entry}\command"),
                    None,
                    &format!("\"{exe}\" \"%V\""),
                )?;
            } else {
                remove(entry)?;
            }
        }
        Ok(())
    }

    fn write(key: &str, name: Option<&str>, value: &str) -> io::Result<()> {
        let key = wide(OsStr::new(key));
        let name = name.map(|name| wide(OsStr::new(name)));
        let data = wide(OsStr::new(value));
        let status = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ref().map_or(ptr::null(), |name| name.as_ptr()),
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * size_of::<u16>()) as u32,
            )
        };
        checked(status)
    }

    fn remove(key: &str) -> io::Result<()> {
        let key = wide(OsStr::new(key));
        match unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, key.as_ptr()) } {
            ERROR_FILE_NOT_FOUND => Ok(()),
            status => checked(status),
        }
    }

    fn checked(status: WIN32_ERROR) -> io::Result<()> {
        match status {
            ERROR_SUCCESS => Ok(()),
            code => Err(io::Error::from_raw_os_error(code as i32)),
        }
    }
}
