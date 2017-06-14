function init()
   width = 80
   height = 40
   cells_horiz = 3
   cells_vert = 3
end

function generate()
   rooms = {}

   function get_grid_size(total_size, divisions)
      local max = math.floor((total_size / divisions) * 0.8)
      local min = math.floor((total_size / divisions) * 0.25)
      if max < 2 then max = 2 end
      if min < 2 then min = 2 end
      return min, max
   end

   minw, maxw = get_grid_size(width, cells_horiz)
   minh, maxh = get_grid_size(height, cells_vert)
   room_min = world.point(minw, minh)
   room_max = world.point(maxw, maxh)

   prefab = Prefab.new(width, height, "water")
   log.info(tostring(prefab))

   cell_width = math.floor(width / cells_horiz)
   cell_height = math.floor(height / cells_vert)

   for i = 1, cells_horiz, 1 do
      rooms[i] = {}
      for j = 1, cells_vert, 1 do
         rooms[i][j] = {rect = world.rect(0, 0, 0, 0), connections = {}, cell = world.point(i, j)}
      end
   end

   function dir_sequence()
      local dirs = {NORTH, WEST, SOUTH, EAST}
      rand.shuffle(dirs)
      return dirs
   end

   function has_value (tab, val)
      for index, value in ipairs(tab) do
         if value == val then
            return true
         end
      end

      return false
   end

   -- TODO: Run until all are connected
   function connect_randomly()
      local connected_cells = {}
      local room, other_room
      local next_cell
      
      local current_cell = world.point(rand.between(1, cells_horiz),
                                       rand.between(1, cells_vert))

      repeat
         local found = false
         local dirs = dir_sequence()

         repeat
            local dir = table.remove(dirs)
            local offset = world.dir(dir)
            next_cell = current_cell + offset
            local bounds = world.rect(1, 1, cells_horiz - 1, cells_vert - 1)

            if bounds:contains(next_cell.x, next_cell.y) then
               room = rooms[current_cell.x][current_cell.y]

               if #room["connections"] > 0 then
                  if has_value(room["connections"], next_cell) then break end
               end

               other_room = rooms[next_cell.x][next_cell.y]

               already_connected = has_value(other_room["connections"], room["cell"])

               if #other_room["connections"] == 0 or not already_connected then
                  table.insert(other_room["connections"], room["cell"])
                  table.insert(room["connections"], other_room["cell"])
                  log.debug("Connected " .. tostring(room["cell"]) .. " to " .. tostring(other_room["cell"]))

                  table.insert(connected_cells, next_cell)

                  current_cell = next_cell
                  found = true
               else
                  log.debug("not found")
               end
            end
         until #dirs == 0 or found == true
      until #dirs == 0
   end

   function connect_neighbors()
      for i = 1, cells_horiz, 1 do
         for j = 1, cells_vert, 1 do
            room = rooms[i][j]

            if #room["connections"] == 0 then
               local dirs = dir_sequence()
               local valid = false

               repeat
                  local dir = table.remove(dirs)
                  local offset = world.dir(dir)
                  local this_cell = world.point(i, j)
                  local next_cell = this_cell + offset

                  local bounds = world.rect(1, 1, cells_horiz - 1, cells_vert - 1)

                  if bounds:contains(next_cell.x, next_cell.y) then
                     other_room = rooms[next_cell.x][next_cell.y]

                     valid = true
                     if #other_room["connections"] == 0 then break end

                     for connected in iter.list_iter(other_room["connections"]) do
                        if connected == this_cell then
                           valid = false
                           break
                        end
                     end

                     if valid then break end
                  end
               until #dirs == 0

               if valid then
                  table.insert(other_room["connections"], room["cell"])
                  table.insert(room["connections"], other_room["cell"])
                  log.debug("Connected " .. tostring(room["cell"]) .. " to " .. tostring(other_room["cell"]))
               else
                  log.warn("Could not connect room! " .. tostring(room["cell"]))
               end
            end
         end
      end
   end

   function all_connected()
      local room = rooms[1][1]
      local visited = {}

      for i = 1, cells_horiz, 1 do
         visited[i] = {}
         for j = 1, cells_vert, 1 do
            visited[i][j] = false
         end
      end

      visited[1][1] = true

      function dfs(room)
         local total = 0
         for cell in iter.list_iter(room["connections"]) do
            if visited[cell.x][cell.y] == false then
               visited[cell.x][cell.y] = true
               total = total + 1 + dfs(rooms[cell.x][cell.y])
            end
         end
         return total
      end
      local total_cells = cells_horiz * cells_vert
      local tot = 1 + dfs(room)
      local done = tot == total_cells

      local asd = "\n"
      for i = 1, cells_horiz, 1 do
         for j = 1, cells_vert, 1 do
            for con in iter.list_iter(rooms[i][j]["connections"]) do
               log.debug(tostring(rooms[i][j]["cell"]) .. " => " .. tostring(con["cell"]))
            end
            if visited[i][j] then
               asd = asd .. " O "
            else
               asd = asd .. " X "
            end
         end
         asd = asd .. "\n"
      end

      log.debug(asd)
      log.debug(tostring(tot))
      return done
   end

   function round(num)
      if num >= 0 then return math.floor(num+.5)
      else return math.ceil(num-.5) end
   end

   function create_rooms()
      local room_size
      local room_pos
      for i = 1, cells_horiz, 1 do
         for j = 1, cells_vert, 1 do
            room_pos = world.point(cell_width * (i-1),
                                   cell_height * (j-1))

            if room_pos.x == 0 then room_pos.x = 1 end
            if room_pos.y == 0 then room_pos.y = 1 end

            room_size = rand.point_between(room_min, room_max)

            if j > 1 then
               other_room = rooms[i][j-1]
               while (room_pos.y - other_room["rect"]:y() + other_room["rect"]:height()) < 3 do
                  room_pos.y = room_pos.y + 1
               end
            end

            if i > 1 then
               other_room = rooms[i-1][j]
               while (room_pos.x - other_room["rect"]:x() + other_room["rect"]:width()) < 3 do
                  room_pos.x = room_pos.x + 1
               end
            end

            log.info(tostring(cell_width) .. " " .. tostring(cell_height) .. " " .. tostring(room_size))
            local offset = world.point(round(rand.zero_to(cell_width - room_size.x)),
                                       round(rand.zero_to(cell_height - room_size.y)))


            while room_pos.x + offset.x + room_size.x >= width do
               if offset.x > 0 then
                  offset.x = offset.x - 1
               else
                  room_size.x = room_size.x - 1
               end
            end

            while room_pos.y + offset.y + room_size.y >= height do
               if offset.y > 0 then
                  offset.y = offset.y - 1
               else
                  room_size.y = room_size.y - 1
               end
            end

            room_pos = room_pos + offset

            for p in iter.rect_iterator(room_pos, room_pos + room_size) do
               room = rooms[i][j]
               room["rect"] = world.rect_from_pts(room_pos, room_size)
               prefab:set(p, "floor")
            end
         end
      end
   end

   function connect_rooms()
      local wall, other_wall

      for i = 1, cells_horiz, 1 do
         for j = 1, cells_vert, 1 do
            room = rooms[i][j]

            for connection in iter.list_iter(room["connections"]) do
               other_room = rooms[connection.x][connection.y]

               if other_room["cell"].x > room["cell"].x then
                  wall = WEST
                  other_wall = EAST
               elseif other_room["cell"].x < room["cell"].x then
                  wall = EAST
                  other_wall = WEST
               elseif other_room["cell"].y < room["cell"].y then
                  wall = NORTH
                  other_wall = SOUTH
               elseif other_room["cell"].y > room["cell"].y then
                  wall = SOUTH
                  other_wall = NORTH
               end

               dig_tunnel(get_wall_position(room, wall), get_wall_position(other_room, other_wall), wall);
            end
         end
      end
   end

   function get_wall_position(room, dir)
      local pos = world.point(0, 0)
      local door, door_pos
      log.info(tostring(room["rect"]))
      if dir == NORTH or dir == SOUTH then
         pos.x = rand.between(room["rect"]:x() + 1, room["rect"]:right() - 2)
         if dir == NORTH then
            pos.y = room["rect"]:y() - 2
            door = pos.y + 1
         else
            pos.y = room["rect"]:bottom() + 1
            door = pos.y
         end
         door_pos = world.point(pos.x, door)
      elseif dir == WEST or dir == EAST then
         pos.y = rand.between(room["rect"]:y() + 1, room["rect"]:bottom() - 2)
         if dir == WEST then
            pos.x = room["rect"]:right() + 1
            door = pos.x
         else
            pos.x = room["rect"]:x() - 2
            door = pos.x + 1
         end
         door_pos = world.point(door, pos.y)
      end

      prefab:set(door_pos, "floor")
      prefab:place_door(door_pos)

      return pos
   end

   function tunnel_sgn(a, b)
      if a > b then
         return -1
      else
         return 1
      end
   end

   function swap(a, b)
      local temp = a.x
      a.x = b.x
      b.x = temp

      temp = a.y
      a.y = b.y
      b.y = temp
   end

   function dig_tunnel(start_pos, end_pos, dir)
      local middle, tunnel_dir
      local tile= "floor"

      log.info("diggin " .. tostring(start_pos) .. " " .. tostring(end_pos) .. " " .. dir)

      if dir == WEST or dir == EAST then
         if start_pos.x > end_pos.x then
            swap(start_pos, end_pos)
         end
         tunnel_dir = tunnel_sgn(start_pos.y, end_pos.y)

         middle = rand.between(start_pos.x, end_pos.x)

         for i = start_pos.x + 1, middle, 1 do
            prefab:set_raw(i, start_pos.y, tile)
         end
         for i = start_pos.y, end_pos.y, tunnel_dir do
            prefab:set_raw(middle, i, tile)
         end
         for i = middle, end_pos.x, 1 do
            prefab:set_raw(i, end_pos.y, tile)
         end
      else
         if start_pos.y > end_pos.y then
            swap(start_pos, end_pos)
         end

         tunnel_dir = tunnel_sgn(start_pos.x, end_pos.x)

         middle = rand.between(start_pos.y, end_pos.y)

         for i = start_pos.y + 1, middle, 1 do
            prefab:set_raw(start_pos.x, i, tile)
         end
         for i = start_pos.x, end_pos.x, tunnel_dir do
            prefab:set_raw(i, middle, tile)
         end
         for i = middle, end_pos.y, 1 do
            prefab:set_raw(end_pos.x, i, tile)
         end
      end
   end

   function add_seawall()
      for pos in prefab:iter() do
         local below_one = world.point(pos.x, pos.y + 1)
         local below_two = world.point(pos.x, pos.y + 2)
         if prefab:in_bounds(below_one) then
            if prefab:get(pos) == "floor" and prefab:get(below_one) == "water" then
               prefab:set(below_one, "seawall")
               if prefab:in_bounds(below_two) then
                  prefab:set(below_two, "seawall")
               end
            end
         end
      end
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
               return prefab:get(pt) == "floor"
         end)
      until point ~= world.point(-1, -1)

      prefab:place_stairs_in(point)
      log.info("stairs at " .. tostring(point))
   end

   i = 0
   repeat
      log.debug("Iterating connections.")
      connect_randomly()
      connect_neighbors()
      i = i + 1
      if i > 100 then
         error("die")
      end
   until all_connected()

   create_rooms()

   for i = 1, cells_horiz, 1 do
      for j = 1, cells_vert, 1 do
         log.info("ROOM " .. tostring(rooms[i][j]["rect"]) .. " " .. i .. " " .. j)
      end
   end

   connect_rooms()
   add_seawall()
   put_stairs()

   return prefab
end
