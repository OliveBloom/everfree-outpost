from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import load
from outpost_data.outpost.lib.palette import recolor, \
        METAL_PALETTES, METAL_STATIONS, METAL_NAMES

from outpost_data.outpost.activity import add_activity_icon

def do_tools(materials):
    base_pal = METAL_PALETTES['_base']

    icons = load('icons/tools.png', unit=ICON_SIZE)
    pick_icon = icons.extract((1, 0))
    axe_icon = icons.extract((3, 0))

    for material in materials:
        pal = METAL_PALETTES[material][2:]
        bar = 'bar/%s' % material if material != 'stone' else 'stone'

        pick = ITEM.new('pick/%s' % material) \
                .display_name('%s Pick' % METAL_NAMES[material]) \
                .icon(recolor(pick_icon, pal))
        RECIPE.from_item(pick) \
                .station(METAL_STATIONS[material]) \
                .inputs({'wood': 5, bar: 5})
        add_activity_icon('item/pick/%s' % material, recolor(pick_icon, pal))

        axe = ITEM.new('axe/%s' % material) \
                .display_name('%s Axe' % METAL_NAMES[material]) \
                .icon(recolor(axe_icon, pal))
        RECIPE.from_item(axe) \
                .station(METAL_STATIONS[material]) \
                .inputs({'wood': 5, bar: 5})
        add_activity_icon('item/axe/%s' % material, recolor(axe_icon, pal))

def init():
    tools = load('icons/tools.png', unit=ICON_SIZE)

    shovel = ITEM.new('shovel').display_name('Shovel').icon(tools.extract((0, 0)))
    RECIPE.from_item(shovel) \
            .station('workbench') \
            .inputs({'wood': 10, 'stone': 10})
    add_activity_icon('item/shovel', tools.extract((0, 0)))

    mallet = ITEM.new('mallet').display_name('Mallet').icon(tools.extract((2, 0)))
    RECIPE.from_item(mallet) \
            .station('workbench') \
            .input('wood', 20)
    add_activity_icon('item/mallet', tools.extract((2, 0)))

    do_tools(('stone', 'copper', 'iron'))
