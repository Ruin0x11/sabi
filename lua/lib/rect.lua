--
-- Copyright 2013 J.C. Moyer
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--   http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.
--

--- Implements a rect object.
-- `rect` is implemented as a table with X, Y, width, and height
-- components residing at the indices `1` to `4` respectively. This design
-- choice has the following implications:
--
-- 1. `unpack` can be used to retrieve the components of a rect as needed.
-- 2. Numerical indices give certain performance benefits due to the way tables
--    are implemented in Lua.
--
-- @type rect
-- @usage
-- local a = rect.new(0, 0, 32, 32)
-- love.graphics.rect('fill', unpack(a))

local rect = {}

rect.__index = rect

local function isrect(t)
  return getmetatable(t) == rect
end

--- Implements binary operator `==` for `rect` objects.
-- @tparam rect a Rect A.
-- @tparam rect b Rect B.
-- @treturn boolean True if the rects are equal; otherwise false.
function rect.__eq(a, b)
  return a[1] == b[1] and a[2] == b[2] and a[3] == b[3] and a[4] == b[4]
end

--- Implements `tostring` for `rect` objects.
-- Do not use this method for serialization as the format may change in the
-- future. This method only guarantees that `rect` objects can be
-- converted to a human-readable representation.
-- @treturn string A `string` representation for this vector.
function rect:__tostring()
  return self[1] .. ',' .. self[2] .. ',' .. self[3] .. ',' .. self[4]
end

--- Constructs a new `rect`.
-- @number[opt=0] x X-coordinate of the rect's top-left point.
-- @number[opt=0] y Y-coordinate of the rect's top-left point.
-- @number[opt=0] w Width of the rect.
-- @number[opt=0] h Height of the rect.
function world.rect(x, y, w, h)
  local instance = { x or 0, y or 0, w or 0, h or 0 }
  setmetatable(instance, rect)
  return instance
end

function world.rect_from_pts(p1, p2)
  local instance = { p1.x or 0, p1.y or 0, p2.x or 0, p2.y or 0 }
  setmetatable(instance, rect)
  return instance
end

--- Returns the X-coordinate of the left side of the rect.
-- @treturn number
function rect:x()
  return self[1]
end

--- Returns the Y-coordinate of the top side of the rect.
-- @treturn number
function rect:y()
  return self[2]
end

--- Returns the width of the rect.
-- @treturn number
function rect:width()
  return self[3]
end

--- Returns the height of the rect.
-- @treturn number
function rect:height()
  return self[4]
end

--- Returns the X-coordinate of the right side of the rect.
-- @treturn number
function rect:right()
  return self[1] + self[3]
end

--- Returns the Y-coordinate of the bottom side of the rect.
-- @treturn number
function rect:bottom()
  return self[2] + self[4]
end

--- Computes the rect formed from the area of two overlapping rects.
-- @tparam rect r Rect to intersect with.
-- @treturn rect|nil If the rects intersect, this function returns
--   the rect formed from the overlapping area between them. If the
--   given rects do not intersect, this function returns `nil`.
function rect:intersect(r)
  local xmin = math.max(self[1], r[1])
  local ymin = math.max(self[2], r[2])
  local xmax = math.min(self:right(), r:right())
  local ymax = math.min(self:bottom(), r:bottom())
  if (xmax < xmin or ymax < ymin) then
    return nil
  end
  return rect.new(xmin, ymin, xmax - xmin, ymax - ymin)
end

--- Performs an intersection test with another rect.
-- @tparam rect r Rect to test with.
-- @treturn bool True if the rects intersect; otherwise, false.
function rect:intersects(r)
  return not (self:bottom() < r[2] or
              self[2] > r:bottom() or
              self[1] > r:right()  or
              self:right() < r[1])
end

--- Computes the rect that contains two given rects.
-- @tparam rect r Rect to compute the union rect with.
-- @treturn rect The rect that contains the given rects.
function rect:union(r)
  local xmin = math.min(self[1], r[1])
  local ymin = math.min(self[2], r[2])
  local xmax = math.max(self:right(), r:right())
  local ymax = math.max(self:bottom(), r:bottom())
  return rect.new(xmin, ymin, xmax - xmin, ymax - ymin)
end

--- Determines whether or not this rect contains a point or rect.
-- @tparam number|rect x If `x` is a number, it is treated as the
--   X-coordinate of a point. If `x` is a `rect`, this function determines
--   whether or not `x` is completely bounded by this rect.
-- @tparam number|nil y Y-coordinate of the point. This parameter is ignored if
--   `x` is a `rect`.
-- @treturn bool True if the point or rect is contained by this rect;
--   otherwise, false.
function rect:contains(x, y)
  if isrect(x) then
    return x[1] >= self[1] and x[2] >= self[2] and
           x:right() <= self:right() and x:bottom() <= self:bottom()
  else
    return x >= self[1] and x <= self:right() and
           y >= self[2] and y <= self:bottom()
  end
end

--- Computes the point that lies in the center of this rect.
-- @treturn[1] number X-coordinate of the point.
-- @treturn[2] number Y-coordinate of the point.
function rect:center()
  return self[1] + self[3] / 2, self[2] + self[4] / 2
end

--- Inflates this rect by the specified amount.
-- The rect is enlarged in both directions on each axis by the exact
-- amount specified. This means that inflating a 20 by 50 rect by 10 and
-- 20 will result in a 40 by 90 rect concentric with the original.
-- @number x Amount to inflate this rect on the X-axis.
-- @number y Amount to inflate this rect on the Y-axis.
function rect:inflate(x, y)
  self[1] = self[1] - x
  self[3] = self[3] + x * 2
  self[2] = self[2] - y
  self[4] = self[4] + y * 2
end

--- Moves this rect by the given vector.
-- @number x Amount to move this rect by on the X-axis.
-- @number y Amount to move this rect by on the Y-axis.
function rect:offset(x, y)
  self[1] = self[1] + x
  self[2] = self[2] + y
end

-- Clones this rect and returns the clone.
function rect:clone()
  return rect.new(unpack(self))
end

return rect
