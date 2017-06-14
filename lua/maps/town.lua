function init()
   streets_width = 3
   blocks_horiz = 3
   blocks_vert = 2
   block_width = rand.between(12, 15)
   block_height = rand.between(12, 15)
end

function generate()
   blocks = {}

   total_width = (streets_width * (blocks_horiz + 1)) + block_width * blocks_horiz
   total_height = (streets_width * (blocks_vert + 1)) + block_height * blocks_vert
   prefab = Prefab.new(total_width, total_height, "sand")

   local i
   for i = 0, blocks_horiz + 1, 1 do
      local skip_height = (streets_width + block_height) * i
      for pos in iter.rect_iterator(world.point(0, skip_height), world.point(total_width, skip_height + streets_width - 1)) do
         prefab:set(pos, "floor")
      end
   end

   for i = 0, blocks_vert + 1, 1 do
      local skip_width = (streets_width + block_width) * i
      for pos in iter.rect_iterator(world.point(skip_width, 0), world.point(skip_width + streets_width - 1, total_height)) do
         prefab:set(pos, "floor")
      end
   end

   return prefab
end
