#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(all(unix, not(target_os = "macos")))]
pub use linux::{backdrop_supported, blur_desktop, set_backdrop};
#[cfg(target_os = "macos")]
pub use macos::{backdrop_supported, set_backdrop};
#[cfg(windows)]
pub use windows::{backdrop_supported, set_backdrop};
