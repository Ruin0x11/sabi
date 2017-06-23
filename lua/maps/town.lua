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
   prefab = Prefab.new(total_width, total_height, "floor")

   local i, j
   local size = world.point(block_width, block_height)

   for j = 0, blocks_vert - 1, 1 do
      for i = 0, blocks_horiz - 1, 1 do
         local start_corner = world.point((i + 1) * streets_width + i * block_width,
            (j + 1) * streets_width + j * block_height)

         for pos in iter.rect_iterator(start_corner, start_corner + size) do
            prefab:set(pos, "sand")
         end

         args = PrefabArgs.new()
         args:set_num("width", block_width - 1)
         args:set_num("height", block_height - 1)
         prefab:deploy_prefab(start_corner + world.point(1, 1), "house", args)
      end
   end

   return prefab
end
