function init()
   width = 10
   height = 10
end

function generate()
   return Prefab.new(width, height, "Floor")
end
