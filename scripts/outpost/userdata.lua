local outpost_ffi = require('outpost_ffi')


-- Userdata extension methods.

function outpost_ffi.types.V3.metatable.__tostring(v)
    return tostring(v:x()) .. ',' .. tostring(v:y()) .. ',' .. tostring(v:z())
end

function outpost_ffi.types.V2.metatable.__tostring(v)
    return tostring(v:x()) .. ',' .. tostring(v:y())
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


-- Don't reuse the same function since Lua makes decisions based on whether the
-- __eq metamethods of the two objects are themselves == or not.
function outpost_ffi.types.Client.metatable.__eq(x, y)
    return x:id() == y:id()
end

function outpost_ffi.types.Entity.metatable.__eq(x, y)
    return x:id() == y:id()
end

function outpost_ffi.types.Structure.metatable.__eq(x, y)
    return x:id() == y:id()
end

function outpost_ffi.types.Inventory.metatable.__eq(x, y)
    return x:id() == y:id()
end


-- Misc methods
function outpost_ffi.types.Client.table.send_message(c, msg)
    c:send_message_raw('***\t' .. msg)
end

function outpost_ffi.types.GenChunk.table.add_structure_with_extras(gc, pos, template, extras)
    local index = gc:add_structure(pos, template)
    for k, v in pairs(extras) do
        gc:set_structure_extra(index, k, v)
    end
end
