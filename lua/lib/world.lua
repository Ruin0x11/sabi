world = {}

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

function world.dir(n)
   local x = dir_table[n][1]
   local y = dir_table[n][2]
   return world.point(x, y)
end
