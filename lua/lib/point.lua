--------------------------------------------------------------------------
-- point.lua
--------------------------------------------------------------------------

local point_metatable = { }

point_metatable.__index = point_metatable

function world.point(x, y)
  local pt = { x = x, y = y }
  setmetatable(pt, point_metatable)
  return pt
end

local function sgn(x)
  if x < 0 then
    return -1
  elseif x > 0 then
    return 1
  else
    return 0
  end
end

function point_metatable:xy()
  return self.x, self.y
end

point_metatable.sgn = function (p)
                        return world.point(sgn(p.x), sgn(p.y))
                      end

point_metatable.__eq = function (a, b)
                         return a.x == b.x and a.y == b.y
                       end

point_metatable.__add = function (a, b)
                          if type(b) == "number" then
                            return world.point(a.x + b, a.y + b)
                          else
                            if a == nil or b == nil then
                              error("Nil points: " .. debug.traceback())
                            end
                            return world.point(a.x + b.x, a.y + b.y)
                          end
                        end

point_metatable.__sub = function (a, b)
                          if type(b) == "number" then
                            return world.point(a.x - b, a.y - b)
                          else
                            return world.point(a.x - b.x, a.y - b.y)
                          end
                        end

point_metatable.__div = function (a, b)
                          if type(b) ~= "number" then
                            error("Can only divide by numbers.")
                          end
                          return world.point(math.floor(a.x / b),
                                           math.floor(a.y / b))
                        end

point_metatable.__mul = function (a, b)
                          if type(b) ~= "number" then
                            error("Can only multiply by numbers.")
                          end
                          return world.point(a.x * b, a.y * b)
                        end

point_metatable.__unm = function (a)
                          return world.point(-a.x, -a.y)
                        end

point_metatable.__tostring = function (p)
                        return "(" .. p.x .. "," .. p.y .. ")"
                      end

point_metatable.__concat = function (pre, post)
                             if getmetatable(pre) == point_metatable then
                               return pre:str() .. post
                             else
                               return pre .. post:str()
                             end
                           end
