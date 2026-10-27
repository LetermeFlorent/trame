use std::io;
use std::{ptr, slice};

use windows_sys::Win32::Security::Credentials::{
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree, CredReadW,
    CredWriteW,
};

use crate::utf16::wide;

pub(super) fn store(key: &str, user: &str, secret: &str) -> io::Result<()> {
    let mut key = wide(key.as_ref());
    let mut user = wide(user.as_ref());
    let mut blob = secret.as_bytes().to_vec();
    let size = u32::try_from(blob.len()).map_err(io::Error::other)?;
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: key.as_mut_ptr(),
        CredentialBlobSize: size,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: user.as_mut_ptr(),
        ..CREDENTIALW::default()
    };
    match unsafe { CredWriteW(&raw const credential, 0) } {
        0 => Err(io::Error::last_os_error()),
        _ => Ok(()),
    }
}

pub(super) fn load(key: &str) -> Option<String> {
    let key = wide(key.as_ref());
    let mut credential: *mut CREDENTIALW = ptr::null_mut();
    if unsafe { CredReadW(key.as_ptr(), CRED_TYPE_GENERIC, 0, &raw mut credential) } == 0 {
        return None;
    }
    let secret = unsafe { blob_text(&*credential) };
    unsafe { CredFree(credential.cast()) };
    secret
}

unsafe fn blob_text(credential: &CREDENTIALW) -> Option<String> {
    if credential.CredentialBlob.is_null() || credential.CredentialBlobSize == 0 {
        return None;
    }
    let size = credential.CredentialBlobSize as usize;
    let bytes = unsafe { slice::from_raw_parts(credential.CredentialBlob, size) };
    String::from_utf8(bytes.to_vec()).ok()
}

pub(super) fn forget(key: &str) {
    let key = wide(key.as_ref());
    unsafe { CredDeleteW(key.as_ptr(), CRED_TYPE_GENERIC, 0) };
}
