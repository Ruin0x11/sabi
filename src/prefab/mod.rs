mod interop;

pub use self::interop::*;

use std::collections::HashMap;
use std::fmt;

use hlua;

use graphics::cell::Cell;
use graphics::Color;
use point::Point;

#[derive(Debug)]
pub enum PrefabError {
    OutOfBounds(i32, i32),
    BadRange(i32, i32),
    LuaException(hlua::LuaError),
    PrefabVarNotDeclared,
}

use self::PrefabError::*;

impl fmt::Display for PrefabError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let string = match *self {
            PrefabError::LuaException(hlua::LuaError::SyntaxError(ref e)) |
            PrefabError::LuaException(hlua::LuaError::ExecutionError(ref e)) => e.clone(),
            ref e => format!("{:?}", e),
        };
        write!(f, "{}", string)
    }
}

impl From<hlua::LuaError> for PrefabError {
    fn from(err: hlua::LuaError) -> PrefabError {
        LuaException(err)
    }
}

pub type PrefabResult<T> = Result<T, PrefabError>;
pub type Markers = HashMap<Point, String>;
pub type PrefabArgs = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Prefab {
    cells: Vec<Cell>,
    size: Point,
    pub markers: Markers,
}

impl fmt::Display for Prefab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;
        for j in 0..self.size.y {
            for i in 0..self.size.x {
                let pos = Point::new(i, j);
                let ch = if self.get(&pos).can_pass_through() {
                    '.'
                } else {
                    '#'
                };
                write!(f, "{}", ch)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

pub fn mark_to_color(mark: &str) -> Color {
    match mark {
        "mob" => Color::new(255, 0, 255),
        "door" => Color::new(0, 0, 255),
        "stairs_in" => Color::new(0, 255, 0),
        "stairs_out" => Color::new(255, 255, 0),
        "connection" => Color::new(255, 0, 0),
        "npc" => Color::new(0, 255, 255),
        _ => Color::new(0, 0, 0),
    }
}

impl<'lua, L> hlua::LuaRead<L> for Prefab
where
    L: hlua::AsMutLua<'lua>,
{
    fn lua_read_at_position(lua: L, index: i32) -> Result<Prefab, L> {
        let val: Result<hlua::UserdataOnStack<Prefab, _>, _> =
            hlua::LuaRead::lua_read_at_position(lua, index);
        val.map(|d| d.clone())
    }
}

impl Prefab {
    pub fn new(x: i32, y: i32, fill: &str) -> Self {
        let mut cells = Vec::new();
        for _ in 0..x {
            for _ in 0..y {
                cells.push(Cell::new(fill));
            }
        }
        Prefab {
            size: Point::new(x, y),
            cells: cells,
            markers: Markers::new(),
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
            let v = &mut self.cells[idx];
            *v = val;
        }
    }

    pub fn get(&self, pt: &Point) -> Cell {
        if self.in_bounds(pt) {
            let idx = self.index(pt);
            self.cells[idx]
        } else {
            Cell::new("nothing")
        }
    }

    pub fn set_marker(&mut self, pt: &Point, val: String) {
        // Only supports one marker per location.
        if self.in_bounds(pt) {
            self.markers.insert(*pt, val);
        }
    }

    pub fn find_marker(&self, query: &str) -> Option<Point> {
        for (point, marker) in self.markers.iter() {
            if *marker == query {
                return Some(*point);
            }
        }
        None
    }

    pub fn size(&self) -> Point {
        self.size
    }

    pub fn width(&self) -> i32 {
        self.size.x
    }

    pub fn height(&self) -> i32 {
        self.size.y
    }

    pub fn merge(&mut self, other: Prefab, offset: Point) {
        for (point, cell) in other.iter() {
            self.set(&(point + offset), *cell);
        }

        for (point, marker) in other.markers {
            self.markers.insert(point + offset, marker);
        }
    }

    pub fn markers<'a>(&'a self) -> impl Iterator<Item = (&'a Point, &'a String)> {
        self.markers.iter()
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
            Some(cell) => Some((level_position, cell)),
            None => None,
        }
    }
}
