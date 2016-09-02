from outpost_data.core import structure
from outpost_data.core.consts import *
from outpost_data.core.builder2 import STRUCTURE
from outpost_data.core.image2 import loader
from outpost_data.outpost.lib import terrain, models

def mk_tree_shapes():
    solid = B_SOLID_SHAPE(S_SOLID) | B_OCCUPIED
    solid2 = solid | B_SUBFLOOR | B_FLOOR
    nil = 0

    tree_arr = [
        nil,    nil,    nil,
        nil,    solid2, nil,
        nil,    nil,    nil,

        nil,    nil,    nil,
        nil,    solid2, nil,
        nil,    nil,    nil,

        solid2, solid2, solid2,
        solid2, solid2, solid2,
        solid2, solid2, solid2,

        solid2, solid2, solid2,
        solid2, solid2, solid2,
        solid2, solid2, solid2,
        ]

    return (structure.Shape(3, 3, 4, tree_arr),
            structure.Shape(3, 3, 1, tree_arr[:9]))


TREE_SHAPE, STUMP_SHAPE = mk_tree_shapes()

def init():
    tiles = loader('tiles', unit=TILE_SIZE)

    terrain.interior_blocks('farmland', tiles('farmland-interior-parts.png'), shape='floor')

    structures = loader('structures', unit=TILE_SIZE)
    s = STRUCTURE.prefixed('tree') \
            .shape(TREE_SHAPE) \
            .layer(1)

    s.new('v0') \
            .part(models.TREE['shadow'], structures('tree-shadow-round.png')) \
            .part(models.TREE['trunk'], structures('tree-trunk.png')) \
            .part(models.TREE['top'], structures('tree-top-round.png'))

    s.new('v1') \
            .part(models.TREE['shadow'], structures('tree-shadow-cone.png')) \
            .part(models.TREE['trunk'], structures('tree-trunk.png')) \
            .part(models.TREE['top'], structures('tree-top-cone.png'))

    STRUCTURE.new('stump') \
            .shape(STUMP_SHAPE) \
            .layer(1) \
            .part(models.TREE['stump'], structures('tree-stump.png'))
