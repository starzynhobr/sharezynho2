pub mod capture;
#[cfg(target_os = "macos")]
mod capture_macos;
#[cfg(windows)]
mod capture_windows;
pub mod receiver;

pub use capture::ScreenCapture;
pub use receiver::ScreenReceiver;
