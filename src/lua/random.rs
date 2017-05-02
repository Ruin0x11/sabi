use prefab::{PrefabError, PrefabResult};
use rand::{self, Rng};
use hlua::{self, Lua};

fn lua_between(a: i32, b: i32) -> PrefabResult<i32> {
    if a == b {
        return Ok(a);
    }
    if a > b {
        return Err(PrefabError::BadRange(a, b))
    }
    Ok(rand::thread_rng().gen_range(a, b))
}

fn lua_zero_to(n: i32) -> PrefabResult<i32> {
    if n <= 0 {
        return Err(PrefabError::BadRange(0, n))
    }
    Ok(rand::thread_rng().gen_range(0, n))
}

fn lua_chance(n: f32) -> bool {
    rand::thread_rng().next_f32() < n
}

fn lua_coinflip() -> bool {
    rand::thread_rng().gen()
}

pub fn add_lua_interop(lua: &mut Lua) {
    let mut rand_namespace = lua.empty_array("rand");

    rand_namespace.set("between", hlua::function2(lua_between));
    rand_namespace.set("zero_to", hlua::function1(lua_zero_to));
    rand_namespace.set("coinflip", hlua::function0(lua_coinflip));
    rand_namespace.set("chance", hlua::function1(lua_chance));
}
