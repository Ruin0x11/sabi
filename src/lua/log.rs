use hlua::{self, Lua};
use slog::Logger;

use log;

make_global!(LUA_LOG, Logger, log::make_logger("lua"));

pub fn lua_log_info(text: String) {
    instance::with(|l| info!(l, text));
}

pub fn lua_log_warn(text: String) {
    instance::with(|l| warn!(l, text));
}

pub fn lua_log_error(text: String) {
    instance::with(|l| error!(l, text));
}

pub fn lua_log_debug(text: String) {
    instance::with(|l| debug!(l, text));
}

pub fn lua_log_trace(text: String) {
    instance::with(|l| trace!(l, text));
}

pub fn add_lua_interop(lua: &mut Lua) {
    let mut log_namespace = lua.empty_array("log");

    log_namespace.set("info", hlua::function1(lua_log_info));
    log_namespace.set("warn", hlua::function1(lua_log_warn));
    log_namespace.set("error", hlua::function1(lua_log_error));
    log_namespace.set("debug", hlua::function1(lua_log_debug));
    log_namespace.set("trace", hlua::function1(lua_log_trace));
}
