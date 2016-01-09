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


-- 'tree' behavior
action.use['tree/v0'] = function(c, s)
    c:pawn():inventory('main'):update('wood', 2)
end
action.use['tree/v1'] = action.use['tree/v0']

tools.handler.axe['tree/v0'] = function(c, s, inv)
    if not ward.check(c, s:pos()) then
        return
    end

    s:replace('stump')
    inv:update('wood', 15)
end
tools.handler.axe['tree/v1'] = tools.handler.axe['tree/v0']

function tools.handler.axe.stump(c, s, inv)
    if not ward.check(c, s:pos()) then
        return
    end

    s:destroy()
    inv:update('wood', 5)
end


-- 'rock' behavior
function action.use.rock(c, s)
    c:pawn():inventory('main'):update('stone', 2)
end

function tools.handler.pick.rock(c, s, inv)
    if not ward.check(c, s:pos()) then
        return
    end

    s:destroy()
    inv:update('stone', 20)
    if math.random() < 0.2 then
        inv:update('crystal', 1)
    end
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


function outpost_ffi.callbacks.login(c)
    c:set_main_inventories(c:pawn():inventory('main'),
                           c:pawn():inventory('ability'))

    -- TODO: would be better to just have an "on register" callback, for
    -- one-time initialization
    if not c:pawn():extra().inited_abilities then
        c:pawn():extra().inited_abilities = true
        if math.floor(c:pawn():get_appearance() / 128) % 2 == 1 then
            c:pawn():inventory('ability'):update('ability/light', 1)
        end
    end
end


function action.use_ability.light(c, inv)
    local val
    if c:pawn():extra().light_active then
        c:pawn():extra().light_active = false
        val = 0
    else
        c:pawn():extra().light_active = true
        val = 0x200
    end
    c:pawn():update_appearance(0x200, val)
end


print('\n\nup and running')
