use std::ffi::CString;

use pgrx::GucSetting;

pub mod client;

pub static PGL_REMOTE_SERVER_URL: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);
