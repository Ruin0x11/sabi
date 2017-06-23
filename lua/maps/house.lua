function init()
   width = rand.between(12, 15)
   height = rand.between(12, 15)
end

   function place_room(rect, flooring)
      for pos in rect:iter() do
         prefab:set(pos, flooring)
      end

      for pos in rect:iter_border() do
         prefab:set(pos, "wall")
      end

      opening = rect:x() + rect:width()/2

      prefab:set(world.point(opening, rect:height()), flooring)
   end

function generate()
   prefab = Prefab.new(width, height, "floor")

   room_width = width - 1
   room_height = height - 1

   main_room = world.rect(0, 0, room_width, room_height)
   place_room(main_room, "tile")

   function place_next_room()
      next_height = math.floor(room_height / 2)
      next_width = math.floor(room_width / 2)
      if rand.coinflip() then
         start_x = prefab:width() - next_width
      else
         start_x = 0
      end
      next_room = world.rect(start_x, 0, next_width, next_height)

      place_room(next_room, "carpet")
      center = world.point(next_room:width() / 2 + start_x, (next_room:height() / 2))
      prefab:place_marker(center, "npc")
   end

   place_next_room()

   return prefab
end
