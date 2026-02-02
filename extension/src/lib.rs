use pgrx::prelude::*;

use std::ffi::CString;

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};

mod datasets;
mod explain;
mod planner;
mod utils;
mod rpc;

pg_module_magic!(name, version);


extension_sql!(
    "CREATE SCHEMA IF NOT EXISTS pgl;
     CREATE TABLE IF NOT EXISTS pgl.pgl_qdataset_status (
         id BIGSERIAL PRIMARY KEY,
         dataset_name VARCHAR(64) UNIQUE,
         current_pos BIGINT
     );",
    name = "create_schema_pglearned"
);

#[pg_guard]
pub extern "C-unwind" fn _PG_init() {
    GucRegistry::define_enum_guc(
        c"pgl.planner_method",
        c"The planner method",
        c"The planner method for pglearned",
        &planner::PGL_PLANNER_METHOD,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_int_guc(
        c"pgl.planner_arm",
        c"The planner arm",
        c"The planner arm for pglearned",
        &planner::PGL_PLANNER_ARM,
        i32::MIN,
        i32::MAX,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_string_guc(
        c"pgl.remote_server_url",
        c"The remote server url",
        c"The remote server url for pglearned",
        &planner::PGL_REMOTE_SERVER_URL,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_enum_guc(
        c"pgl.planner_mode",
        c"The planner mode",
        c"The planner mode for pglearned",
        &planner::PGL_PLANNER_MODE,
        GucContext::Userset,
        GucFlags::default(),
    );

    unsafe {
        explain::register();
        planner::register();
    }
}
