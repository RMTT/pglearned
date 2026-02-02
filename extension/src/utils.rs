use pgrx::pg_sys;
use std::ffi::CString;

/// Sets a GUC configuration option using the PostgreSQL C API.
///
/// # Arguments
/// * `name` - The name of the GUC variable.
/// * `value` - The value to set.
/// * `action` - The GucAction (e.g., GUC_ACTION_LOCAL, GUC_ACTION_SET).
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(anyhow::Error)` if setting the option fails.
pub fn set_config_internal(
    name: &str,
    value: &str,
    action: pgrx::pg_sys::GucAction::Type,
) -> anyhow::Result<()> {
    let name_c = CString::new(name).map_err(|e| anyhow::anyhow!("Invalid GUC name: {}", e))?;
    let value_c = CString::new(value).map_err(|e| anyhow::anyhow!("Invalid GUC value: {}", e))?;

    unsafe {
        let result = pg_sys::set_config_option(
            name_c.as_ptr(),
            value_c.as_ptr(),
            pg_sys::GucContext::PGC_USERSET,
            pg_sys::GucSource::PGC_S_SESSION,
            action,
            true,  // changeVal
            0,     // elevel (0 = don't log errors, return 0 on failure)
            false, // is_reload
        );

        if result == 0 {
            anyhow::bail!("Failed to set GUC '{}' to '{}'", name, value);
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn set_config(name: &str, value: &str) -> anyhow::Result<()> {
    set_config_internal(name, value, pg_sys::GucAction::GUC_ACTION_SET)
}

pub fn set_config_local(name: &str, value: &str) -> anyhow::Result<()> {
    set_config_internal(name, value, pg_sys::GucAction::GUC_ACTION_LOCAL)
}
