use std::ffi::{OsStr, OsString, c_void};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
    CoInitializeEx, CoTaskMemFree, CoUninitialize,
};
use windows_sys::Win32::UI::Shell::{
    FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
};
use windows_sys::core::{GUID, HRESULT, PCWSTR, PWSTR};

use super::FolderRequest;

const FILE_OPEN_DIALOG: GUID = GUID::from_u128(0xdc1c5a9c_e88a_4dde_a5a1_60f82a20aef7);
const FILE_DIALOG: GUID = GUID::from_u128(0x42f85136_db7e_439c_85f1_e4075d135fc8);
const SHELL_ITEM: GUID = GUID::from_u128(0x43826d1e_e718_42ee_bc55_a1e261c37bfe);

const RELEASE: usize = 2;
const SHOW: usize = 3;
const SET_OPTIONS: usize = 9;
const GET_OPTIONS: usize = 10;
const SET_FOLDER: usize = 12;
const SET_TITLE: usize = 17;
const GET_RESULT: usize = 20;
const GET_DISPLAY_NAME: usize = 5;

type Release = unsafe extern "system" fn(*mut c_void) -> u32;
type Show = unsafe extern "system" fn(*mut c_void, HWND) -> HRESULT;
type SetOptions = unsafe extern "system" fn(*mut c_void, u32) -> HRESULT;
type GetOptions = unsafe extern "system" fn(*mut c_void, *mut u32) -> HRESULT;
type SetItem = unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT;
type SetText = unsafe extern "system" fn(*mut c_void, PCWSTR) -> HRESULT;
type GetItem = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT;
type GetName = unsafe extern "system" fn(*mut c_void, i32, *mut PWSTR) -> HRESULT;

struct Apartment;

impl Apartment {
    fn enter() -> Option<Self> {
        let mode = COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE;
        let result = unsafe { CoInitializeEx(ptr::null(), mode as u32) };
        (result >= 0).then_some(Self)
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct Object(*mut c_void);

impl Object {
    fn from_out(fill: impl FnOnce(*mut *mut c_void) -> HRESULT) -> Option<Self> {
        let mut raw = ptr::null_mut();
        let result = fill(&mut raw);
        (result >= 0 && !raw.is_null()).then_some(Self(raw))
    }

    fn method<F: Copy>(&self, slot: usize) -> F {
        unsafe {
            let table = *self.0.cast::<*const F>();
            *table.add(slot)
        }
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        let release: Release = self.method(RELEASE);
        unsafe { release(self.0) };
    }
}

pub(super) fn choose(request: &FolderRequest) -> Option<PathBuf> {
    let _apartment = Apartment::enter()?;
    let dialog = Object::from_out(|out| unsafe {
        CoCreateInstance(
            &FILE_OPEN_DIALOG,
            ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &FILE_DIALOG,
            out,
        )
    })?;
    prepare(&dialog, request);
    let owner = request.owner.unwrap_or_default() as HWND;
    let show: Show = dialog.method(SHOW);
    if unsafe { show(dialog.0, owner) } < 0 {
        return None;
    }
    let get_result: GetItem = dialog.method(GET_RESULT);
    let item = Object::from_out(|out| unsafe { get_result(dialog.0, out) })?;
    path_of(&item)
}

fn prepare(dialog: &Object, request: &FolderRequest) {
    let get_options: GetOptions = dialog.method(GET_OPTIONS);
    let set_options: SetOptions = dialog.method(SET_OPTIONS);
    let set_title: SetText = dialog.method(SET_TITLE);
    let title = wide(request.title.as_ref());
    let mut options = 0;
    unsafe {
        get_options(dialog.0, &mut options);
        set_options(dialog.0, options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM);
        set_title(dialog.0, title.as_ptr());
    }
    let Some(start) = request.start.as_deref().and_then(shell_item) else {
        return;
    };
    let set_folder: SetItem = dialog.method(SET_FOLDER);
    unsafe { set_folder(dialog.0, start.0) };
}

fn shell_item(folder: &Path) -> Option<Object> {
    let path = wide(folder.as_os_str());
    Object::from_out(|out| unsafe {
        SHCreateItemFromParsingName(path.as_ptr(), ptr::null_mut(), &SHELL_ITEM, out)
    })
}

fn path_of(item: &Object) -> Option<PathBuf> {
    let get_name: GetName = item.method(GET_DISPLAY_NAME);
    let mut name: PWSTR = ptr::null_mut();
    if unsafe { get_name(item.0, SIGDN_FILESYSPATH, &mut name) } < 0 || name.is_null() {
        return None;
    }
    let length = (0..)
        .take_while(|index| unsafe { *name.add(*index) } != 0)
        .count();
    let units = unsafe { std::slice::from_raw_parts(name, length) };
    let path = PathBuf::from(OsString::from_wide(units));
    unsafe { CoTaskMemFree(name.cast()) };
    Some(path)
}

fn wide(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain([0]).collect()
}
