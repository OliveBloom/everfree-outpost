local action = require('outpost.action')
local structure_items = require('structure_items')
local tools = require('tools')

function action.use.teleporter(c, s)
    c:pawn():teleport(s:extra().destination)
end

function action.use_item.teleporter(c, inv)
    local home = c:extra().home_pos
    if home == nil then
        c:send_message('Must /sethome before placing teleporter')
        return
    end
    local s = structure_items.use_item(c, inv, 'teleporter', 'teleporter')
    s:extra().destination = home
end

function tools.handler.pick.teleporter(c, s, inv)
    structure_items.use_structure(c, s, 'teleporter')
end


function action.use.dungeon_entrance(c, s)
    c:send_message('Not yet implemented')
end

