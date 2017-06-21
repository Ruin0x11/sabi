function init()
   cell_size = 7
   cells_h = 8
   cells_v = 8
end

function generate()
   width = (cell_size + 1) * cells_h + 1
   height = (cell_size + 1) * cells_v + 1

   prefab = Prefab.new(width, height, "floor")

   local i, j
   for j = 0, cells_v, 1 do
      for i = 0, cells_h, 1 do
         local sx = i * (cell_size + 1)
         local sy = j * (cell_size + 1)
         local start_pos = world.point(sx, sy)
         local size = world.point(cell_size + 1)
         local end_pos = start_pos + size
         for pos in iter.border_iterator(start_pos, end_pos) do
            prefab:set(pos, "wall")
         end
      end
   end

   for j = 0, cells_v, 1 do
      for i = 0, cells_h, 1 do
         local sx = i * (cell_size + 1)
         local sy = j * (cell_size + 1)
         local start_pos = world.point(sx, sy)

         local halfway = (cell_size + 2 - 1) / 2
         local center = start_pos + world.point(halfway)

         if j ~= cells_v - 1 then
            prefab:set(center + (world.dir(SOUTH) * halfway), "floor")
         end
         if i ~= cells_h - 1 then
            prefab:set(center + (world.dir(EAST) * halfway), "floor")
         end
         prefab:set(center, "sand")
      end
   end

   prefab:place_marker(world.point(2, 2), "stairs_in")

   return prefab
end
