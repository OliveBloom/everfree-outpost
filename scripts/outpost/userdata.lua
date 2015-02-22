local outpost_ffi = require('outpost_ffi')


-- Userdata extension methods.

function outpost_ffi.types.V3.metatable.__tostring(v)
    return tostring(v:x()) .. ',' .. tostring(v:y()) .. ',' .. tostring(v:z())
end

function outpost_ffi.types.World.metatable.__tostring(x)
    return 'World'
end

function outpost_ffi.types.Client.metatable.__tostring(x)
    return 'Client:' .. tostring(x:id())
end

function outpost_ffi.types.Entity.metatable.__tostring(x)
    return 'Entity:' .. tostring(x:id())
end

function outpost_ffi.types.Structure.metatable.__tostring(x)
    return 'Structure:' .. tostring(x:id())
end

function outpost_ffi.types.Inventory.metatable.__tostring(x)
    return 'Inventory:' .. tostring(x:id())
end


function outpost_ffi.types.StableClient.metatable.__tostring(x)
    return 'StableClient:' .. x:id()
end

function outpost_ffi.types.StableEntity.metatable.__tostring(x)
    return 'StableEntity:' .. x:id()
end

function outpost_ffi.types.StableStructure.metatable.__tostring(x)
    return 'StableStructure:' .. x:id()
end

function outpost_ffi.types.StableInventory.metatable.__tostring(x)
    return 'StableInventory:' .. x:id()
end
