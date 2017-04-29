mod interop;

pub use self::interop::{map_from_prefab, add_lua_interop};

use std::collections::HashMap;

use hlua;

use graphics::cell::{self, Cell, StairDir};
use point::{Point, POINT_ZERO, RectangleIter};

#[derive(Debug)]
pub enum PrefabError {
    CellTypeNotFound(String),
    OutOfBounds(i32, i32),
    BadRange(i32, i32),
    LuaException(hlua::LuaError),
    PrefabVarNotDeclared,
}

use self::PrefabError::*;

impl From<hlua::LuaError>for PrefabError {
    fn from(err: hlua::LuaError) -> PrefabError {
        LuaException(err)
    }
}

pub type PrefabResult<T> = Result<T, PrefabError>;

#[derive(Debug, Clone)]
pub struct Prefab {
    cells: Vec<Cell>,
    size: Point,
    markers: HashMap<Point, PrefabMarker>,
}

#[derive(Debug, Clone)]
pub enum PrefabMarker {
    Mob(String),
    Door,
    Stairs(StairDir)
}

impl<'lua, L> hlua::LuaRead<L> for Prefab
    where L: hlua::AsMutLua<'lua>
{
    fn lua_read_at_position(lua: L, index: i32) -> Result<Prefab, L> {
        let val: Result<hlua::UserdataOnStack<Prefab, _>, _> =
            hlua::LuaRead::lua_read_at_position(lua, index);
        val.map(|d| d.clone())
    }
}

impl Prefab {
    pub fn new(x: i32, y: i32, fill: Cell) -> Self {
        let mut cells = Vec::new();
        for _ in 0..x {
            for _ in 0..y {
                cells.push(fill);
            }
        }
        Prefab {
            size: Point::new(x, y),
            cells: cells,
            markers: HashMap::new(),
        }
    }

    pub fn in_bounds(&self, pt: &Point) -> bool {
        *pt >= Point::new(0, 0) && *pt < self.size
    }

    pub fn set(&mut self, pt: &Point, val: Cell) {
        if self.in_bounds(pt) {
            let idx = (pt.y * self.size.x + pt.x) as usize;
            let mut v = self.cells.get_mut(idx).unwrap();
            *v = val;
        }
    }

    pub fn get(&self, pt: &Point) -> Cell {
        if self.in_bounds(pt) {
            let idx = (pt.y * self.size.x + pt.x) as usize;
            self.cells.get(idx).unwrap().clone()
        } else {
            cell::NOTHING
        }
    }

    pub fn width(&self) -> i32 {
        self.size.x
    }

    pub fn height(&self) -> i32 {
        self.size.y
    }

    pub fn iter(&self) -> PrefabIter {
        PrefabIter {
            index: 0,
            width: self.width(),
            height: self.height(),
            inner: self.cells.iter(),
        }
    }
}

// FIXME: duplication from chunk
pub struct PrefabIter<'a> {
    index: i32,
    width: i32,
    height: i32,
    inner: ::std::slice::Iter<'a, Cell>,
}

impl<'a> Iterator for PrefabIter<'a> {
    type Item = (Point, &'a Cell);

    fn next(&mut self) -> Option<(Point, &'a Cell)> {
        let x = self.index % self.width;
        let y = self.index / self.height;
        let level_position = Point::new(x, y);
        self.index += 1;
        match self.inner.next() {
            Some(cell) => {
                Some((level_position, cell))
            }
            None => None,
        }
    }
}
