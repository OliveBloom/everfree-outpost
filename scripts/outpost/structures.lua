local autorotate = require('outpost.lib.autorotate')
local mallet = require('outpost.lib.mallet')
local structure_items = require('outpost.lib.structure_items')
local door = require('outpost.lib.door')

local add_structure_item = structure_items.add_structure_item
local add_attachment_item = structure_items.add_attachment_item
local mallet_cycle = mallet.mallet_cycle




add_structure_item('statue', 'statue/e')
mallet_cycle('statue/', { 'e', 's', 'w', 'n' })

add_structure_item('bed')
add_structure_item('table')
add_structure_item('trophy')
add_structure_item('fountain')
add_structure_item('torch')
add_structure_item('stair', 'stair/n')



local horiz_walls = {
    ['house_wall/edge/horiz/in'] = true,
    ['house_wall/edge/horiz/out'] = true,
    ['house_wall/tee/n/in'] = true,
    ['house_wall/tee/n/out'] = true,
    ['wood_wall/edge/horiz'] = true,
    ['wood_wall/tee/n'] = true,
    ['stone_wall/edge/horiz'] = true,
    ['stone_wall/tee/n'] = true,
    ['cottage_wall/edge/horiz'] = true,
    ['cottage_wall/variant/v0'] = true,
    ['cottage_wall/variant/v1'] = true,
    ['cottage_wall/variant/v2'] = true,
    ['cottage_wall/tee/n'] = true,
}

-- NB: Other `cabinets` setup is in `object.chest`.
structure_items.attachment_map['cabinets'] = horiz_walls
structure_items.attachment_map['bookshelf/0'] = horiz_walls

add_attachment_item('bookshelf', 'bookshelf/0')


add_structure_item('wood_pillar', 'pillar/wood')
add_structure_item('stone_pillar', 'pillar/stone')
