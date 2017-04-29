iter = {}

------------------------------------------------------------------------------
-- rectangle_iterator
------------------------------------------------------------------------------
iter.rectangle_iterator = {}

function iter.rectangle_iterator:_new ()
  local m = {}
  setmetatable(m, self)
  self.__index = self
  return m
end

function iter.rectangle_iterator:new (corner1, corner2, filter, rvi)
  if corner1 == nil or corner2 == nil then
    error("need two corners to a rectangle")
  end

  if corner2.x < corner1.x then
    error("corner2's x less than corner1's; did you swap them?")
  elseif corner2.y < corner1.y then
    error("corner2's y less than corner1's; did you swap them?")
  end

  local mt = iter.rectangle_iterator:_new()
  mt.min_x = corner1.x
  mt.min_y = corner1.y
  mt.max_x = corner2.x
  mt.max_y = corner2.y
  mt.cur_x = corner1.x - 1
  mt.cur_y = corner1.y
  mt.filter = filter or nil
  mt.rvi = rvi or false

  return mt:iter()
end

function iter.rectangle_iterator:next()
  local point = nil
  repeat
    if self.cur_x >= self.max_x then
      self.cur_y = self.cur_y + 1
      self.cur_x = self.min_x
      if self.cur_y > self.max_y then
        point = -1
      else
        point = self:check_filter(world.point(self.cur_x, self.cur_y))
        if point == nil then
          point = -2
        end
      end
    else
      self.cur_x = self.cur_x + 1
      point = self:check_filter(world.point(self.cur_x, self.cur_y))
      if point == nil then
        point = -2
      end
    end
    if point == -1 then
      return nil
    elseif point == -2 then
      point = nil
    end
  until point

  return point
end

function iter.rectangle_iterator:check_filter(point)
  if self.filter ~= nil then
    if self.filter(point) then
      if self.rvi then
        return self.filter(point)
      else
        return point
      end
    else
      return nil
    end
  else
    return point
  end
end

function iter.rectangle_iterator:iter ()
  return function() return self:next() end, nil, nil
end

function iter.rect_iterator(top_corner, bottom_corner, filter, rvi)
  return iter.rectangle_iterator:new(top_corner, bottom_corner, filter, rvi)
end

function iter.rect_size_iterator(top_corner, size, filter, rvi)
  return iter.rect_iterator(top_corner, top_corner + size - world.point(1, 1),
                            filter, rvi)
end

function iter.mons_rect_iterator (top_corner, bottom_corner, filter)
  return iter.rect_iterator(top_corner, bottom_corner, iter.monster_filter(filter), true)
end

function iter.border_iterator(top_corner, bottom_corner, rvi)
  local function check_inner(point)
    if point.x == top_corner.x or point.x == bottom_corner.x or point.y == top_corner.y or point.y == bottom_corner.y then
      return point
    end
    return nil
  end
  return iter.rectangle_iterator:new(top_corner, bottom_corner, check_inner, rvi)
end

function iter.map_iterator(filter, rvi)
   return iter.rect_iterator(world.point(0, 0), world.size(), filter, rvi)
end

-------------------------------------------------------------------------------
-- Circle_iterator
-------------------------------------------------------------------------------

function iter.circle_iterator (radius, ic, filter, center, rvi)
  if radius == nil then
    error("circle_iterator needs a radius")
  end

  local y_x, y_y = you.pos()

  if center ~= nil then
    y_x, y_y = center:xy()
  end

  local include_center = ic
  local top_corner = world.point(y_x - radius, y_y - radius)
  local bottom_corner = world.point(y_x + radius, y_y + radius)

  local function check_dist (point)
    local _x, _y = point:xy()
    local dist = world.distance(y_x, y_y, _x, _y)
    local npoint = nil

    if filter ~= nil then
      if rvi then
        npoint = filter(point)
      end

      if not filter(point) then
        return nil
      end
    end

    if y_x == _x and y_y == _y then
      if rvi and include_center then
        return npoint
      else
        return include_center
      end
    end

    if dist >= (radius * radius) + 1 then
      return nil
    end

    if rvi then
      return npoint
    else
      return true
    end
  end

  return iter.rect_iterator(top_corner, bottom_corner, check_dist, rvi)
end

-------------------------------------------------------------------------------
-- adjacent_iterator
-------------------------------------------------------------------------------

function iter.adjacent_iterator (ic, filter, center, rvi)
  local y_x, y_y = world.point(0, 0)

  if center ~= nil then
    y_x, y_y = center:xy()
  end

  local top_corner = world.point(y_x - 1, y_y - 1)
  local bottom_corner = world.point(y_x + 1, y_y + 1)
  local include_center = ic

  local function check_adj (point)
    local _x, _y = point:xy()
    local npoint = point

    if filter ~= nil then
      if rvi then
        npoint = filter(point)
      end

      if not filter(point) then
        return nil
      end
    end

    if y_x == _x and y_y == _y then
      if rvi and include_center then
        return npoint
      else
        return include_center
      end
    end

    if rvi then
      return npoint
    else
      return true
    end
  end

  return iter.rect_iterator(top_corner, bottom_corner, check_adj, rvi)
end

function iter.mons_adjacent_iterator (ic, filter, center)
  return iter.adjacent_iterator(ic, iter.monster_filter(filter), center, true)
end

function iter.adjacent_iterator_to(center, include_center, filter)
  return iter.adjacent_iterator(include_center, filter, center, true)
end
