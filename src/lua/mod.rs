pub mod log;
mod random;
pub use self::log::*;

use std::fs::File;
use std::io::Read;

use hlua::{self, Lua};

use prefab;

make_global!(LUA_CONTEXT, Lua<'static>, init());

pub use self::instance::*;

const SCRIPT_DIRECTORY: &str = "lua";

pub fn run_script<'a, 'lua>(lua: &'a mut Lua<'lua>, filename: &str) -> Result<(), hlua::LuaError>
           {
    let mut script = String::new();
    let full_path = format!("{}/{}.lua", SCRIPT_DIRECTORY, filename);
    File::open(full_path).expect("No such script file").read_to_string(&mut script).unwrap();
    lua.execute::<()>(&script)
}

pub fn run_script_and_return<'a, 'lua, T>(lua: &'a mut Lua<'lua>,
                                      filename: &str,
                                      var_name: &str) -> Result<Option<T>, hlua::LuaError>
    where T: hlua::LuaRead<hlua::PushGuard<&'a mut hlua::Lua<'lua>>>,
{
    run_script(lua, filename)?;
    Ok(lua.get(var_name))
}

fn open_libs<'a>(lua: &'a mut Lua) -> Result<(), hlua::LuaError> {
    lua.openlibs();

    self::log::add_lua_interop(lua);
    self::random::add_lua_interop(lua);
    prefab::add_lua_interop(lua);

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
