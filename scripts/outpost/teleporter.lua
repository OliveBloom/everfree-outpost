local action = require('core.action')
local structure_items = require('outpost.lib.structure_items')
local tools = require('outpost.lib.tools')


function action.use.dungeon_entrance(c, s)
    if s:extra().target_plane == nil then
        local p = s:world():create_plane('Dungeon')
        p:extra().exit_pos = c:pawn():pos()
        s:extra().target_plane = p:stable_id()
    end

    local entrance_pos = V3.new(128, 128, 12) * V3.new(32, 32, 32)
    c:pawn():teleport_stable_plane(s:extra().target_plane, entrance_pos)
end


function action.use.dungeon_exit(c, s)
    local p = c:pawn():plane()
    c:pawn():teleport_stable_plane(c:world():get_forest_plane(), p:extra().exit_pos)
end
