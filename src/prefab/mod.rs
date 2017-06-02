mod interop;

pub use self::interop::{map_from_prefab, add_lua_interop};

use std::collections::HashMap;

use hlua;

use graphics::cell::{self, Cell};
use point::{Point};

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
    pub markers: HashMap<Point, PrefabMarker>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrefabMarker {
    Mob(String),
    Door,
    StairsIn,
    StairsOut,
    Connection
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

    fn index(&self, pt: &Point) -> usize {
        (pt.y * self.size.x + pt.x) as usize
    }

    pub fn in_bounds(&self, pt: &Point) -> bool {
        *pt >= Point::new(0, 0) && *pt < self.size
    }

    pub fn set(&mut self, pt: &Point, val: Cell) {
        if self.in_bounds(pt) {
            let idx = self.index(pt);
            let mut v = &mut self.cells[idx];
            *v = val;
        }
    }

    pub fn get(&self, pt: &Point) -> Cell {
        if self.in_bounds(pt) {
            let idx = self.index(pt);
            self.cells[idx]
        } else {
            cell::NOTHING
        }
    }

    pub fn set_marker(&mut self, pt: &Point, val: PrefabMarker) {
        // Only supports one marker per location.
        if self.in_bounds(pt) {
            self.markers.insert(*pt, val);
        }
    }

    pub fn find_marker(&self, query: PrefabMarker) -> Option<Point> {
        for (point, marker) in self.markers.iter() {
            if *marker == query {
                return Some(*point);
            }
        }
        None
    }

    pub fn width(&self) -> i32 {
        self.size.x
    }

    pub fn height(&self) -> i32 {
        self.size.y
    }

    pub fn combine(&mut self, other: &mut Prefab, x: i32, y: i32) {
        let offset = Point::new(x, y);
        for (point, cell) in other.iter() {
            self.set(&(point + offset), *cell);
        }
    }

    pub fn iter(&self) -> PrefabIter {
        PrefabIter {
            index: 0,
            width: self.width(),
            inner: self.cells.iter(),
        }
    }
}

#[cfg(never)]
impl Prefab {
    fn connected(&self, from: &Point, to: &Point) -> bool {
        let blocked = |pos| self.get(pos).can_pass_through();
        Path::find(self, from, to, blocked).len() > 0
    }
}

// FIXME: duplication from chunk
pub struct PrefabIter<'a> {
    index: i32,
    width: i32,
    inner: ::std::slice::Iter<'a, Cell>,
}

impl<'a> Iterator for PrefabIter<'a> {
    type Item = (Point, &'a Cell);

    fn next(&mut self) -> Option<(Point, &'a Cell)> {
        let x = self.index % self.width;
        let y = self.index / self.width;
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
