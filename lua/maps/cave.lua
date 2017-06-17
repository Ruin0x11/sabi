function init()
   width = 50
   height = 50
end

function generate()
   prefab = Prefab.new(width, height, "wall")
   local dood = 7

   local i, j
   for j = 0, height, 1 do
      for i = 0, width, 1 do
         prefab:set_raw(i, j, "floor")

         if i > dood and j > dood and i + 1 < width - dood and j + 1 < height - dood then
            prefab:set_raw(i, j, "water")
         end
      end
   end

   prefab:place_stairs_in_raw(1, 1)

   if true then
      dood = 9 + rand.zero_to(3)
      for j = 0, height - dood * 2, 1 do
         local ja = dood + j
         for i = 0, width - dood * 2, 1 do
            local ia = dood + i
            prefab:set_raw(ia, ja, "sand")
         end
      end
   end

   return prefab
end
