function init()
    width = rand.between(12, 15)
    height = rand.between(12, 15)
end

function generate()
    prefab = Prefab.new(width, height, "floor")

    room_width = width - 1
    room_height = height - 1

    function place_room(rect)
        for pos in rect:iter() do
            prefab:set(pos, "tile")
        end

        for pos in rect:iter_border() do
            prefab:set(pos, "wall")
        end

        opening = rect:width()/2

        prefab:set(world.point(opening, rect:height()), "tile")

        center = world.point(opening, (rect:height() / 2))

        log.debug(tostring(center))
        prefab:place_marker(center, "npc")
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

    return prefab
end
