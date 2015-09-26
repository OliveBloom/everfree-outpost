local action = require('core.action')
local timer = require('outpost.ext.timer')
local tools = require('outpost.lib.tools')
local util = require('core.util')
local ward = require('outpost.lib.ward')

local function handler(c, s, inv)
    if not ward.check(c, s:pos()) then
        return
    end
    s:destroy()
    inv:update('stone', 2)
end

tools.handler.pick['cave_junk/0'] = handler
tools.handler.pick['cave_junk/1'] = handler
tools.handler.pick['cave_junk/2'] = handler


tools.handler.pick._ = function(c, s, inv)
    local pos = util.hit_tile(c:pawn())
    if not ward.check(c, pos) then
        return
    end

    local plane = c:pawn():plane()
    local mined, err = plane:set_cave(pos)
    if err then
        print('error mining at', pos, err)
    end
    if mined then
        inv:update('pick', -1)
        inv:update('stone', 20)
    end
end


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

action.use['dungeon/door/key/closed'] = function(c, s)
    if not use_key(c:pawn():inventory('main')) then
        c:send_message('You need a key to open this door.')
        return
    end

    s:replace('dungeon/door/key/opening')
    s:set_timer(500)
end

timer.handler['dungeon/door/key/opening'] = function(s)
    s:replace('dungeon/door/key/open')
end
