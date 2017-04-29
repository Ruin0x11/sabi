use hlua::{self, Lua};

use point::Point;
use graphics::Glyph;
use graphics::cell::{Cell, CellType};
use prefab::*;
use lua;

const PREFAB_VARIABLE: &str = "prefab";

pub fn map_from_prefab<'a>(lua: &'a mut Lua, name: &str) -> PrefabResult<Prefab> {
    let map_filename = &format!("maps/{}", name);

    lua.execute::<()>("prefab = Prefab.new(10, 10, \"Wall\")")?;

    match lua::run_script_and_return(lua, map_filename, PREFAB_VARIABLE)? {
        Some(prefab) => Ok(prefab),
        None         => Err(PrefabError::PrefabVarNotDeclared),
    }
}

pub fn lua_new(x: i32, y: i32, fill: String) -> PrefabResult<Prefab> {
    let cell_type = match fill.parse::<CellType>() {
        Ok(t)     => t,
        Err(_)    => return Err(CellTypeNotFound(fill)),
    };

    let cell = Cell {
        type_: cell_type,
        glyph: Glyph::Wall,
        feature: None,
    };

    Ok(Prefab::new(x, y, cell))
}

pub fn lua_get(prefab: &Prefab, x: i32, y: i32) -> PrefabResult<String> {
    let pt = Point::new(x, y);
    if !prefab.in_bounds(&pt) {
        return Err(OutOfBounds(x, y));
    }
    Ok(format!("{:?}", prefab.get(&pt).type_))
}

pub fn lua_set(prefab: &mut Prefab, x: i32, y: i32, name: String) -> PrefabResult<()> {
    let cell_type = match name.parse::<CellType>() {
        Ok(t)     => t,
        Err(_)    => return Err(CellTypeNotFound(name)),
    };

    let cell = Cell {
        type_: cell_type,
        glyph: Glyph::Wall,
        feature: None,
    };

    prefab.set(&Point::new(x, y), cell);
    Ok(())
}

pub fn lua_blocked(prefab: &Prefab, x: i32, y: i32) -> bool {
    let pt = Point::new(x, y);
    if !prefab.in_bounds(&pt) {
        return true
    }
    !prefab.get(&pt).can_pass_through()
}

pub fn lua_in_bounds(prefab: &Prefab, x: i32, y: i32) -> bool {
    let pt = Point::new(x, y);
    prefab.in_bounds(&pt)
}

pub fn lua_width(prefab: &Prefab) -> i32 {
    prefab.width()
}

pub fn lua_height(prefab: &Prefab) -> i32 {
    prefab.height()
}

pub fn add_lua_interop(lua: &mut Lua) {
    let mut prefab_namespace = lua.empty_array("Prefab");

    prefab_namespace.set("new", hlua::function3(lua_new));
}

// this macro implements the required trait so that we can *push* the object to lua
// (ie. move it inside lua)
implement_lua_push!(Prefab, |mut metatable| {
    let mut index = metatable.empty_array("__index");

    index.set("set_raw", hlua::function4(lua_set));
    index.set("get_raw", hlua::function3(lua_get));
    index.set("blocked_raw", hlua::function3(lua_blocked));
    index.set("in_bounds_raw", hlua::function3(lua_in_bounds));

    index.set("width", hlua::function1(lua_width));
    index.set("height", hlua::function1(lua_height));
});

// this macro implements the require traits so that we can *read* the object back
implement_lua_read!(Prefab);
