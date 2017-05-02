-- Extends userdata with methods provided by the given metatable.
function extend(userdata, methods_metatable)
   local mt = getmetatable(userdata)
   for k,v in pairs(methods_metatable) do mt.__index[k] = v end
   return userdata
end
