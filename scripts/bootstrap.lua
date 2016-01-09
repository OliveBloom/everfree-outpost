-- Override print to output to stderr.  stdout is used for communication with
-- the server wrapper.
function print(...)
    local s = ''
    for i = 1, select('#', ...) do
        local x = select(i, ...)
        s = s .. tostring(x) .. '\t'
    end
    io.stderr:write(s .. '\n')
end

local function dump_rec(x, n)
    for k,v in pairs(x) do
        if type(v) == 'table' then
            print(n .. tostring(k) .. ':')
            dump_rec(v, n .. '  ')
        else
            print(n .. tostring(k) .. ': ' .. tostring(v))
        end
    end
end

local function dump(x)
    if type(x) == 'table' then
        dump_rec(x, '')
    else
        print(x)
    end
end

package.loaded.bootstrap = {
    dump = dump,
}


-- Put some type tables in global scope
V3 = outpost_ffi.types.V3.table
V2 = outpost_ffi.types.V2.table
World = outpost_ffi.types.World.table

ExtraArg = outpost_ffi.types.ExtraArg.table

Time = outpost_ffi.types.Time.table
Timer = outpost_ffi.types.Timer.table


require('core.userdata')
require('core.extra')
require('core.eval')
require('core.timer')
local action = require('core.action')
local util = require('core.util')

require('loader')


-- TODO: move the rest of this stuff into outpost/ somewhere
local tools = require('outpost.lib.tools')
local ward = require('outpost.lib.ward')

-- No 'local' so it gets exposed to repl scripts
trigger = require('outpost.trigger')



function action.open_inventory(c)
    c:open_inventory(c:pawn():inventory('main'))
end



-- Commands
local spawn_point = V3.new(32, 32, 0)
PLANE_FOREST = 'Everfree Forest'

function check_forest(client)
    if client:pawn():plane():name() ~= PLANE_FOREST then
        client:send_message("That doesn't work here.")
        return false
    else
        return true
    end
end


no_op = function(...) end


function client_by_name(s)
    local w = World.get()
    for i = 0, 100 do
        local c = w:get_client(i)
        if c ~= nil and c:name() == s then
            return c
        end
    end
end


print('\n\nup and running')
