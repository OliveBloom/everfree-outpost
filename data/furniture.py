from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.geom import Mesh
from outpost_data.core.image2 import loader
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes



BED_MESH = Mesh(
        meshes.quad_y(64,  8, 56,  0, 10) +
        meshes.quad_z(10,  8, 56,  2, 64) +
        meshes.quad_y( 2,  8, 56, 10, 20))

SHELF_MESH = Mesh(
        meshes.quad_y(29,  0, 32,  0, 60) +
        meshes.quad_z(60,  0, 32, 20, 29))


STAIR_N_MESH = Mesh([tuple(x * TILE_SIZE for x in v)
    for v in meshes.quad((0, 1, 0), (1, 1, 0), (1, 0, 1), (0, 0, 1))])


def do_bed(image):
    s = STRUCTURE.new('bed') \
            .mesh(BED_MESH) \
            .shape(structure.solid(2, 1, 2)) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s).display_name('Bed')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 20)

def do_table(image):
    s = STRUCTURE.new('table') \
            .mesh(meshes.solid(2, 2, 1)) \
            .shape(structure.solid(2, 2, 1)) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s).display_name('Table')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 20)

def do_statue(image):
    s = STRUCTURE.prefixed('statue') \
            .mesh(meshes.solid(2, 1, 2)) \
            .shape(structure.solid(2, 1, 2)) \
            .layer(1)

    image = image.with_unit((64, 96))
    s.new('n').image(image.extract((0, 0)))
    s.new('s').image(image.extract((1, 0)))
    s.new('e').image(image.extract((2, 0)))
    s.new('w').image(image.extract((3, 0)))

    i = ITEM.from_structure(s['e'], name='statue').display_name('Statue')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('stone', 50)

def do_shelves(image):
    image = image.with_unit((32, 96))

    s_base = STRUCTURE.child() \
            .mesh(SHELF_MESH) \
            .shape(structure.solid(1, 1, 2)) \
            .layer(2)

    s = s_base.new('cabinets').image(image.extract((0, 0)))
    i = ITEM.from_structure(s, extract_offset=(0, 16)).display_name('Cabinets')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 20)

    s = s_base.prefixed('bookshelf')
    s.new('0').image(image.extract((1, 0)))
    s.new('1').image(image.extract((2, 0)))
    s.new('2').image(image.extract((3, 0)))
    i = ITEM.from_structure(s['0'], name='bookshelf', extract_offset=(0, 16)) \
            .display_name('Bookshelves')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 20)

def do_trophy(image):
    s = STRUCTURE.new('trophy') \
            .mesh(meshes.solid(1, 1, 1)) \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s).display_name('Trophy')

def do_fountain(image):
    s = STRUCTURE.new('fountain') \
            .mesh(meshes.solid(2, 2, 1)) \
            .shape(structure.solid(2, 2, 1)) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s).display_name('Fountain')

def do_stair(image):
    s = STRUCTURE.new('stair/n') \
            .mesh(STAIR_N_MESH) \
            .shape(structure.Shape(1, 1, 1, ['ramp_n'])) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s, name='stair').display_name('Stairs')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 10)

def do_torch(image, variant, variant_disp, color):
    name = 'torch/' + variant if variant is not None else 'torch'
    disp_name = variant_disp + ' Torch' if variant is not None else 'Torch'

    s = STRUCTURE.new(name) \
            .mesh(meshes.solid(1, 1, 1)) \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1) \
            .anim([image.extract((i, 0), (1, 2)) for i in range(4)], 4) \
            .light((16, 16, 32), color, 300)
    i = ITEM.from_structure(s).display_name(disp_name)
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 2) \
            .input('stone', 1)


def init():
    tiles = loader('tiles', unit=TILE_SIZE)
    icons = loader('icons', unit=TILE_SIZE)
    structures = loader('structures', unit=TILE_SIZE)

    # Bed and table

    furniture = structures('furniture.png')

    do_bed(furniture.extract((0, 0), (2, 3)))
    do_table(furniture.extract((2, 0), (2, 3)))
    do_statue(structures('statue.png'))
    do_shelves(furniture.extract((4, 0), (4, 3)))
    do_trophy(structures('trophy.png'))
    do_fountain(structures('fountain.png'))
    do_stair(structures('stair.png'))

    ITEM.new('book') \
            .display_name('Book') \
            .icon(icons('gervais_roguelike/AngbandTk_book.png'))

    do_torch(structures('torch.png'), None, None, (255, 230, 200))
