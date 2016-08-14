from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.geom import Mesh
from outpost_data.core.image2 import load
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes

CRATE_MESH = Mesh(
        meshes.quad_y(31,  0, 32,  0, 20) +
        meshes.quad_z(20,  0, 32,  0, 31))

VARIANTS = (
        'pepper',
        'carrot',
        'artichoke',
        'cucumber',
        'potato',
        'tomato',
        'corn',
        )

def init():
    crate_sheet = load('structures/crate.png', unit=TILE_SIZE)

    sb = STRUCTURE.child() \
            .mesh(CRATE_MESH) \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1)
    s = sb.new('crate').image(crate_sheet.extract((0, 0), size=(1, 2)))
    i = ITEM.from_structure(s).display_name('Crate')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input('wood', 20)

    sb = sb.prefixed('crate')
    for i, v in enumerate(VARIANTS):
        img = crate_sheet.extract((i + 1, 0), size=(1, 2))
        sb.new(v).image(img)
