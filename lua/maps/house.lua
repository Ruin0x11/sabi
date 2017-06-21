function init()
   width = rand.between(12, 15)
   height = rand.between(12, 15)
end

function generate()
   prefab = Prefab.new(width, height, "floor")

   room_width = width - 1
   room_height = height - 1

   function place_room(rect, flooring)
      for pos in rect:iter() do
         prefab:set(pos, flooring)
      end

      for pos in rect:iter_border() do
         prefab:set(pos, "wall")
      end

      opening = rect:width()/2

      prefab:set(world.point(opening, rect:height()), flooring)
   end

   main_room = world.rect(0, 0, room_width, room_height)
   place_room(main_room, "tile")

   center = world.point(main_room:width() / 2, (main_room:height() / 2))
   prefab:place_marker(center, "npc")

   function place_next_room()
      next_height = room_height / 2
      next_width = rand.between(room_width / 3, room_width)
      start_x = rand.between(0, room_width)
      next_room = world.rect(start_x, 0, next_width, next_height)

      place_room(next_room, "carpet")
   end

   place_next_room()

   return prefab
end
