function rand.dir4()
   local dir = rand.zero_to(4) * 2
   return world.dir(dir)
end

function rand.point_between(lo, hi)
   return world.point(rand.between(lo.x, hi.x),
                      rand.between(lo.y, hi.y))
end

function rand.point_zero_to(hi)
   return rand.point_between(world.point(0, 0), hi)
end

function rand.shuffle(t)
    assert( t, "rand.shuffle() expected a table, got nil" )
    local iterations = #t
    local j

    for i = iterations, 2, -1 do
        j = rand.zero_to(i)
        t[i], t[j] = t[j], t[i]
    end
end

