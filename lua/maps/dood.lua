function init()
   width = 50
   height = 50
   caviness_min = 8
   caviness_max = 10
end

function put_stairs()
   local point
   local i = 0
   repeat
      i = i + 1
      if i > 100 then
         error("die")
      end

      point = prefab:random_point(function(pt)
            return not prefab:blocked(pt)
      end)
   until point ~= world.point(-1, -1)

   prefab:place_stairs_in(point)
   log.info("stairs at " .. tostring(point))
end

-- TODO: ensure there isn't already a stair there

function put_stairs_b()
   local point
   local i = 0
   repeat
      i = i + 1
      if i > 100 then
         error("die")
      end

      point = prefab:random_point(function(pt)
            return not prefab:blocked(pt)
      end)
   until point ~= world.point(-1, -1)

   prefab:place_stairs_out(point)
   log.info("stairs at " .. tostring(point))
end


function carve_circle(kind, dood)
   while true do
      local rx = rand.zero_to(width)
      local ry = rand.zero_to(height)
      if prefab:get_raw(rx, ry) ~= "wall" then
         goto nexta
      end

      local loops = rand.between(caviness_min, caviness_max)

      for j = 0, loops, 1 do
         local py = j + ry - loops / 2

         for k = 0, loops, 1 do
            local px = k + rx - loops / 2

            if px < 1 or py < 1 or px >= width - 1 or py >= height - 1 then
               goto nextb
            end

            if world.dist(world.point(rx, ry), world.point(px, py)) >= loops / 2 then
               goto nextb
            end

            for _, dir in pairs(world.dir_table) do
               local s = world.point(px + dir[1], py + dir[2])
               if prefab:get(s) ~= kind then
                  if prefab:get(s) ~= "floor" then
                     if prefab:get(s) ~= "wall" then
                        goto nextb
                     end
                  end
               end
            end

            prefab:set_raw(px, py, kind)
            ::nextb::
         end
      end
      break
      ::nexta::
   end
end

function generate()
   local i, j, k
   local kind = "sand"

   prefab = Prefab.new(width, height, "wall")

   for i = 0, 50, 1 do
      local dood = 100 + i + 1
      prefab:print()

      carve_circle(kind, dood)

   end
   for pos in iter.rect_iterator(world.point(0, 0), world.point(height / 2 - 2, width / 2 - 2)) do
      if prefab:get(pos) ~= "wall" then
         goto nextc
      end

      if prefab:get(pos) ~= "floor" then
         goto nextc
      end

      function check(pos, dirs)
         if prefab.get(pos + world.dir(dirs[0])) == "wall" then
            if prefab.get(pos + world.dir(dirs[1])) == "wall" then
               if prefab.get(pos + world.dir(dirs[2])) == kind then
                  if prefab.get(pos + world.dir(dirs[3])) == kind then
                     prefab.set(pos, "water")
                  end
               end
               return true
            end
         end
         return false
      end

      if check(pos, { EAST, WEST, NORTH, SOUTH }) then
         goto nextc
      end

      if check(pos, { NORTH, SOUTH, EAST, WEST }) then
         goto nextc
      end

      ::nextc::
   end

   put_stairs()
   put_stairs_b()

   return prefab
end
