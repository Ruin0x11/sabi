world = {}

function Prefab:set(point, terrain)
   return self:set_raw(point.x, point.y, terrain)
end

function Prefab:get(point)
   return self:get_raw(point.x, point.y)
end

function Prefab:blocked(point)
   return self:blocked_raw(point.x, point.y)
end

function Prefab:in_bounds(point)
   return self:in_bounds_raw(point.x, point.y)
end

function world.size()
   return world.point(self:width() - 1, self:height() - 1)
end
