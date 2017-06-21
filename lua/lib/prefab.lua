local prefab_metatable = {}

function Prefab.new(w, h, fill)
   local prefab = Prefab.new_raw(w, h, fill)
   prefab = extend(prefab, prefab_metatable)
   return prefab
end

function prefab_metatable:size()
   return world.point(self:width(), self:height())
end

function prefab_metatable:set(point, terrain)
   return self:set_raw(point.x, point.y, terrain)
end

function prefab_metatable:get(point)
   return self:get_raw(point.x, point.y)
end

function prefab_metatable:blocked(point)
   return self:blocked_raw(point.x, point.y)
end

function prefab_metatable:in_bounds(point)
   return self:in_bounds_raw(point.x, point.y)
end

function prefab_metatable:place_marker(point, marker)
   return self:place_marker_raw(point.x, point.y, marker)
end

function prefab_metatable:deploy_prefab(point, kind, args)
   return self:deploy_prefab_raw(point.x, point.y, kind, args)
end

function prefab_metatable:iter()
   return iter.rect_iterator(world.point(0, 0), self:size())
end

function prefab_metatable:random_point(filter)
   local found = false
   local point = world.point(-1, -1)
   local iterations = 100
   repeat
      iterations = iterations - 1
      point = rand.point_zero_to(self:size())
      if filter(point) == true then
         found = true
      end
   until found or iterations <= 0

   return point
end
