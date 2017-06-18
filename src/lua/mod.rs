pub mod log;
mod random;
pub use self::log::*;
mod world;

use std::fs::File;
use std::io::Read;

use hlua::{self, Lua};

use prefab;

make_global!(LUA_CONTEXT, Lua<'static>, init());

pub use self::instance::*;

const SCRIPT_DIRECTORY: &str = "lua";

pub fn log(mes: String) {
    self::log::lua_log_debug(mes);
}

pub fn run_script<'a, 'lua>(lua: &'a mut Lua<'lua>, filename: &str) -> Result<(), hlua::LuaError>
{
    let mut script = String::new();
    let full_path = format!("{}/{}.lua", SCRIPT_DIRECTORY, filename);
    File::open(full_path).expect("No such script file").read_to_string(&mut script).unwrap();
    lua.execute::<()>(&script)
}

fn open_libs(lua: &mut Lua) -> Result<(), hlua::LuaError> {
    lua.openlibs();

    self::log::add_lua_interop(lua);
    self::random::add_lua_interop(lua);
    self::world::add_lua_interop(lua);
    prefab::add_lua_interop(lua);

    open_lua_libs(lua)
}

fn open_lua_libs(lua: &mut Lua) -> Result<(), hlua::LuaError> {
    run_script(lua, "lib/init")
}

fn init() -> Lua<'static> {
    let mut lua = Lua::new();
    open_libs(&mut lua).unwrap();
    lua
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua() {
        init();
    }
}
