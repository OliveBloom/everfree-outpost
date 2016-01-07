local action = require('core.action')
local timer = require('outpost.ext.timer')
local tools = require('outpost.lib.tools')
local util = require('core.util')
local ward = require('outpost.lib.ward')
local door = require('outpost.lib.door')


function use_key(inv)
    if inv:count('key/master') > 0 then
        return true
    end
    if inv:count('key') > 0 then
        inv:update('key', -1)
        return true
    end
    return false
end


door.register_anims('dungeon/door/key', 500)
door.register_anims('dungeon/door/puzzle', 500)

action.use['dungeon/door/key/closed'] = function(c, s)
    if not use_key(c:pawn():inventory('main')) then
        c:send_message('You need a key to open this door.')
        return
    end

    door.open(s)
end
