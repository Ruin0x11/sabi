function rand.dir4()
   local dir = rand.zero_to(4)
   if dir == 0 then return wrl.point(0, -1) end
   if dir == 1 then return wrl.point(-1, 0) end
   if dir == 2 then return wrl.point(1,  0) end
   if dir == 3 then return wrl.point(0,  1) end
   return world.point(0, 0)
end
