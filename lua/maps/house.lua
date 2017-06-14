width = rand.between(15, 18)
height = rand.between(15, 18)

prefab = Prefab.new(width, height, "floor")

room_width = rand.between(width - 8, width - 4)
room_height = rand.between(height - 8, height - 4)

function place_room(rect)
   for pos in rect:iter() do
      prefab:set(pos, "Tile")
   end

   for pos in rect:iter() do
      prefab:set(pos, "Wall")
   end
end

main_room = world.rect(0, 0, room_width, room_height)

place_room(main_room)

function place_next_room()
   h_min = height - room_height
   if h_min <= 2 then
      return
   end
   next_height = rand.between(2, h_min)
   next_width = rand.between(room_width / 3, room_width)
   start_x = rand.between(0, room_width)
   next_room = world.rect(start_x, room_height, next_width, next_height)

   place_room(next_room)
end

prefab:place_stairs_in(world.point(1, 1))
