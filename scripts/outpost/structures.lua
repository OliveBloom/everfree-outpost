local structure_items = require('outpost.lib.structure_items')

local add_structure_item = structure_items.add_structure_item
local add_attachment_item = structure_items.add_attachment_item



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
