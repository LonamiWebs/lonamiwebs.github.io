#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::{DirectoryWatcher, watch};

#[cfg(not(target_os = "windows"))]
mod not_windows;

#[cfg(not(target_os = "windows"))]
pub use not_windows::{DirectoryWatcher, watch};
