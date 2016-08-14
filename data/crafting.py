from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.geom import Mesh
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes


def do_workbench(image):
    s = STRUCTURE.new('workbench') \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.solid(1, 1, 1)) \
            .image(image) \
            .layer(1)
    i = ITEM.from_structure(s).display_name('Workbench')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input('wood', 10)

def do_furnace(image):
    s = STRUCTURE.new('furnace') \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.solid(1, 1, 1)) \
            .image(image) \
            .layer(1)
    i = ITEM.from_structure(s).display_name('Furnace')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input('stone', 10)

def do_anvil(image):
    s = STRUCTURE.new('anvil') \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.front(1, 1, 1)) \
            .image(image) \
            .layer(1)
    i = ITEM.from_structure(s).display_name('Anvil')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input('wood', 5) \
            .input('stone', 5) \
            .input('bar/iron', 5)


def init():
    icons = loader('icons', unit=ICON_SIZE)
    structures = loader('structures', unit=TILE_SIZE)

    do_workbench(structures('workbench.png'))
    do_furnace(structures('furnace.png').extract((0, 0), size=(1, 2)))
    do_anvil(structures('anvil.png'))
