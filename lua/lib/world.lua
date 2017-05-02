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

dir_name_table =
   {

   }

dir_table =
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
   local x = dir_table[n][1]
   local y = dir_table[n][2]
   return world.point(x, y)
end
