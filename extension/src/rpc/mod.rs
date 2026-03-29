use std::ffi::CString;

use pgrx::GucSetting;

pub mod client;

pub static PGL_REMOTE_SERVER_URL: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);

pub fn remote_server_url() -> Option<String> {
    PGL_REMOTE_SERVER_URL
        .get()
        .map(|url| url.to_string_lossy().into_owned())
        .filter(|url| !url.trim().is_empty())
}
