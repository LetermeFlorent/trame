use std::ffi::c_void;
use std::ptr;

use windows_sys::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};
use windows_sys::core::HRESULT;

const RELEASE: usize = 2;

type Release = unsafe extern "system" fn(*mut c_void) -> u32;

pub(super) struct Apartment;

impl Apartment {
    pub(super) fn enter() -> Option<Self> {
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

pub(super) struct Object(pub(super) *mut c_void);

impl Object {
    pub(super) fn from_out(fill: impl FnOnce(*mut *mut c_void) -> HRESULT) -> Option<Self> {
        let mut raw = ptr::null_mut();
        let result = fill(&mut raw);
        (result >= 0 && !raw.is_null()).then_some(Self(raw))
    }

    pub(super) fn method<F: Copy>(&self, slot: usize) -> F {
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
