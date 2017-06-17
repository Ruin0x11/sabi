use point::Point;
use prefab::PrefabResult;
use hlua::{self, Lua};

fn lua_dist(ax: i32, ay: i32, bx: i32, by: i32) -> PrefabResult<f32> {
    let a = Point::new(ax, ay);
    let b = Point::new(bx, by);
    Ok(a.distance(b))
}

pub fn add_lua_interop(lua: &mut Lua) {
    let mut world_namespace = lua.empty_array("world");

    world_namespace.set("dist_raw", hlua::function4(lua_dist));
}
