from outpost_data.core import structure
from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import loader

from outpost_data.outpost.lib import meshes
from outpost_data.outpost.lib.palette import METAL_PALETTES, METAL_NAMES, recolor

def init():
    icons = loader('icons', unit=ICON_SIZE)
    structures = loader('structures', unit=TILE_SIZE)

    ore_vein = structures('ore-vein.png')
    ore = icons('stones.png').extract((1, 0))
    bar = icons('metal-bar.png')

    s_ore_vein = STRUCTURE.prefixed('ore_vein') \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.front(1, 1, 1)) \
            .layer(1)
    i_ore = ITEM.prefixed('ore')
    i_bar = ITEM.prefixed('bar')
    r_bar = RECIPE.child().station('furnace')

    base_pal = METAL_PALETTES['_base']

    for m in ('copper', 'iron'):
        pal = METAL_PALETTES[m]

        s_ore_vein.new(m).image(recolor(ore_vein, pal, base_pal))

        i1 = i_ore.new(m) \
                .display_name('%s Ore' % METAL_NAMES[m]) \
                .icon(recolor(ore, pal, base_pal))
        i2 = i_bar.new(m) \
                .display_name('%s Bar' % METAL_NAMES[m]) \
                .icon(recolor(bar, pal, base_pal))
        r_bar.from_item(i2).input(i1.unwrap().name, 1)

