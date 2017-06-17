-- table "world" was predeclared in lua/world.rs

function world.size()
   return world.point(self:width() - 1, self:height() - 1)
end

NORTH = "North"
NORTHWEST = "Northwest"
WEST = "West"
SOUTHWEST = "Southwest"
SOUTH = "South"
SOUTHEAST = "Southeast"
EAST = "East"
NORTHEAST = "Northeast"

world.dir_table =
{
      [NORTH]     = { 0,  -1 } ,
      [NORTHWEST] = { -1, -1 } ,
      [WEST]      = { -1,  0 } ,
      [SOUTHWEST] = { -1,  1 } ,
      [SOUTH]     = { 0,   1 } ,
      [SOUTHEAST] = { 1,   1 } ,
      [EAST]      = { 1,   0 } ,
      [NORTHEAST] = { 1,  -1 } ,
}

function world.dir(n)
   local x = world.dir_table[n][1]
   local y = world.dir_table[n][2]
   return world.point(x, y)
end

function world.dist(a, b)
   return world.dist_raw(a.x, a.y, b.x, b.y)
end
