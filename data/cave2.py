from outpost_data.core import image2
from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes


def do_cave_junk(img):
    sb = STRUCTURE.prefixed('cave_junk') \
            .mesh(meshes.front(1, 1, 1)) \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1)
    for i in range(3):
        sb.new(str(i)).image(img.extract((i, 0)))


def init():
    tiles = loader('tiles', unit=TILE_SIZE)
    structures = loader('structures', unit=TILE_SIZE)

    do_cave_junk(structures('cave-junk.png'))
