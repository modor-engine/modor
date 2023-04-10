#[cfg(all(unix, not(apple), not(target_os = "android")))]
include!("linux.rs");

#[cfg(target_os = "windows")]
include!("windows.rs");

#[cfg(not(any(
    all(unix, not(apple), not(target_os = "android")),
    target_os = "windows"
)))]
include!("not_linux_windows.rs");
