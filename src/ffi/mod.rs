pub use self::list::List;
pub use self::version::version;

pub mod consts;
pub mod easy;
pub mod err;
pub mod info;
pub mod list;
pub mod opt;
pub mod version;

// On OSX, curl is shipped by default and also doesn't have a pkg-config file,
// so we just hardcode linking directly to `curl`
#[cfg(target_os = "macos")]
#[link(name = "curl")]
extern {}

// On windows, pkg-config doesn't work very well, so we pick up a custom-built
// libcurl from curl-sys. We also want to make sure it's pulled in statically.
#[cfg(windows)]
#[link(name = "ws2_32")]
#[link(name = "curl", kind = "static")]
extern {}

// Everywhere else, we just let pkg-config figure it out.
#[cfg(all(not(windows), not(target_os = "macos")))]
link_config!("libcurl", ["favor_static"])
