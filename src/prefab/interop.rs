use glob;
use hlua::{self, Lua};

use point::Point;
use graphics::cell::Cell;
use prefab;
use prefab::*;
use lua;

pub fn get_prefab_names() -> Vec<String> {
    let mut names = Vec::new();
    for entry in glob::glob("lua/maps/*.lua").expect("No prefab path!") {
        if let Ok(path) = entry {
            names.push(path.file_stem().unwrap().to_str().unwrap().to_owned());
        }
    }
    names
}

pub fn create(name: &str, args: &Option<PrefabArgs>) -> PrefabResult<Prefab> {
    lua::log(format!("Starting creation of prefab \"{}\"", name));

    let res = lua::with_mut(|l| map_from_prefab(l, name, args));

    lua::log(format!("Finished creating prefab \"{}\"", name));

    res
}

pub fn build_prefab_args(args: &PrefabArgs) -> String {
    let mut res = String::new();
    for (k, v) in args.iter() {
        res.push_str(&format!("{} = {}\n", k, v));
    }
    res
}

pub fn map_from_prefab<'a>(lua: &'a mut Lua, name: &str, args: &Option<PrefabArgs>) -> PrefabResult<Prefab> {
    let map_filename = &format!("maps/{}", name);

    lua.execute::<()>("prefab = Prefab.new(1, 1, \"floor\")")?;
    lua.execute::<()>("function init(); error(\"prefab init() not declared!\"); end")?;
    lua.execute::<()>("function generate(); error(\"prefab generate() not declared!\"); end")?;

    lua::run_script(lua, map_filename)?;
    lua.execute::<()>("init()")?;

    if let Some(ref args) = *args {
        let args_script = build_prefab_args(&args);
        println!("args_script: {:?}", args_script);
        lua.execute::<()>(&args_script)?;
    }

    lua.execute::<()>("prefab = generate()")?;

    lua.get("prefab").ok_or_else(|| PrefabError::PrefabVarNotDeclared)
}

pub fn lua_new(x: i32, y: i32, fill: String) -> PrefabResult<Prefab> {
    Ok(Prefab::new(x, y, &fill))
}

fn lua_get(prefab: &Prefab, x: i32, y: i32) -> PrefabResult<String> {
    let pt = Point::new(x, y);
    if !prefab.in_bounds(&pt) {
        return Err(OutOfBounds(x, y));
    }
    Ok(format!("{}", prefab.get(&pt).name()))
}

fn lua_set(prefab: &mut Prefab, x: i32, y: i32, cell_type: String) -> PrefabResult<()> {
    prefab.set(&Point::new(x, y), Cell::new(&cell_type));
    Ok(())
}

fn lua_blocked(prefab: &Prefab, x: i32, y: i32) -> bool {
    let pt = Point::new(x, y);
    if !prefab.in_bounds(&pt) {
        return true
    }
    !prefab.get(&pt).can_pass_through()
}

fn lua_in_bounds(prefab: &Prefab, x: i32, y: i32) -> bool {
    let pt = Point::new(x, y);
    prefab.in_bounds(&pt)
}

fn lua_width(prefab: &Prefab) -> i32 {
    prefab.width()
}

fn lua_height(prefab: &Prefab) -> i32 {
    prefab.height()
}

fn lua_print(prefab: &Prefab) {
    lua::lua_log_info(format!("{}", prefab));
}

fn lua_place_marker(prefab: &mut Prefab, x: i32, y: i32, marker: String) {
    let pt = Point::new(x, y);
    prefab.set_marker(&pt, marker);
}

fn lua_deploy_prefab(prefab: &mut Prefab, x: i32, y: i32, kind: String, args: PrefabArgsLua) -> Result<(), PrefabError> {
    let other = map_from_prefab(&mut lua::init(), &kind, &Some(args.0))?;
    prefab.merge(other, Point::new(x, y));

    Ok(())
}

#[derive(Clone)]
struct PrefabArgsLua(PrefabArgs);

impl<'lua, L> hlua::LuaRead<L> for PrefabArgsLua
    where L: hlua::AsMutLua<'lua>
{
    fn lua_read_at_position(lua: L, index: i32) -> Result<PrefabArgsLua, L> {
        let val: Result<hlua::UserdataOnStack<PrefabArgsLua, _>, _> =
            hlua::LuaRead::lua_read_at_position(lua, index);
        val.map(|d| d.clone())
    }
}

fn lua_prefab_args_new() -> PrefabArgsLua {
    PrefabArgsLua(PrefabArgs::new())
}

fn lua_prefab_args_set(prefab_args: &mut PrefabArgsLua, key: String, val: String) {
    prefab_args.0.insert(key, val);
}

fn lua_prefab_args_set_num(prefab_args: &mut PrefabArgsLua, key: String, val: i32) {
    prefab_args.0.insert(key, val.to_string());
}


pub fn add_lua_interop(lua: &mut Lua) {
    {
        let mut prefab_namespace = lua.empty_array("Prefab");
        prefab_namespace.set("new_raw", hlua::function3(lua_new));
    }

    let mut prefab_args_namespace = lua.empty_array("PrefabArgs");
    prefab_args_namespace.set("new", hlua::function0(lua_prefab_args_new));
}


implement_lua_push!(Prefab, |mut metatable| {
    let mut index = metatable.empty_array("__index");

    index.set("set_raw", hlua::function4(lua_set));
    index.set("get_raw", hlua::function3(lua_get));
    index.set("blocked_raw", hlua::function3(lua_blocked));
    index.set("in_bounds_raw", hlua::function3(lua_in_bounds));
    index.set("place_marker_raw", hlua::function4(lua_place_marker));
    index.set("deploy_prefab_raw", hlua::function5(lua_deploy_prefab));

    index.set("width", hlua::function1(lua_width));
    index.set("height", hlua::function1(lua_height));

    index.set("print", hlua::function1(lua_print));
});

implement_lua_read!(Prefab);


implement_lua_push!(PrefabArgsLua, |mut metatable| {
    let mut index = metatable.empty_array("__index");

    index.set("set", hlua::function3(lua_prefab_args_set));
    index.set("set_num", hlua::function3(lua_prefab_args_set_num));
});

implement_lua_read!(PrefabArgsLua);


#[cfg(test)]
mod tests {
    use prefab;

    #[test]
    fn test_prefab_args() {
        let args = prefab_args! {
            width: 80,
            height: 40,
        };

        let created = prefab::create("blank", &Some(args)).unwrap();

        assert_eq!(created.width(), 80);
        assert_eq!(created.height(), 40);
    }
}
