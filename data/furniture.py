from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.geom import Mesh
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes



BED_MESH = Mesh(
        meshes.quad_y(64,  8, 56,  0, 10) +
        meshes.quad_z(10,  8, 56,  2, 64) +
        meshes.quad_y( 2,  8, 56, 10, 20))

SHELF_MESH = Mesh(
        meshes.quad_y(29,  0, 32,  0, 60) +
        meshes.quad_z(60,  0, 32, 20, 29))

BUREAU_MESH = Mesh(
        meshes.quad_y(24,  0, 32,  0, 24) +
        meshes.quad_z(24,  0, 32,  9, 24))

WIDE_BUREAU_MESH = Mesh(
        meshes.quad_y(30,  0, 64,  0, 15) +
        meshes.quad_z(15,  0, 64,  6, 30))


STAIR_N_MESH = Mesh([tuple(x * TILE_SIZE for x in v)
    for v in meshes.quad((0, 1, 0), (1, 1, 0), (1, 0, 1), (0, 0, 1))])


def unpack_variant(v, default_material, default_amount):
    if v is None:
        return ('', '', default_material, default_amount)
    else:
        name, disp_name, material, amount = v
        return (name + '/', disp_name + ' ', material, amount)

def do_bed(image, variant=None):
    prefix, disp_prefix, material, amount = unpack_variant(variant, 'wood', 20)

    s = STRUCTURE.new(prefix + 'bed') \
            .mesh(BED_MESH) \
            .shape(structure.solid(2, 2, 1)) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s).display_name(disp_prefix + 'Bed')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input(material, amount)

def do_table(image, variant=None):
    prefix, disp_prefix, material, amount = unpack_variant(variant, 'wood', 20)

    s = STRUCTURE.new(prefix + 'table') \
            .mesh(meshes.solid(2, 2, 1)) \
            .shape(structure.solid(2, 2, 1)) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s).display_name(disp_prefix + 'Table')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input(material, amount)

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
            .station('workbench') \
            .input('stone', 50)

def do_shelf_collider():
    STRUCTURE.new('shelf_collider') \
            .parts(()) \
            .shape(structure.solid(1, 1, 2)) \
            .layer(1)

def do_shelves(image, variant=None):
    prefix, disp_prefix, material, amount = unpack_variant(variant, 'wood', 20)

    image = image.with_unit((32, 96))

    s_base = STRUCTURE.child() \
            .mesh(SHELF_MESH) \
            .shape(structure.empty(1, 1, 2)) \
            .layer(2)

    s = s_base.new(prefix + 'cabinets').image(image.extract((0, 0)))
    i = ITEM.from_structure(s, extract_offset=(0, 16)).display_name(disp_prefix + 'Cabinets')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input(material, amount)

    s = s_base.prefixed(prefix + 'bookshelf')
    s.new('0').image(image.extract((1, 0)))
    s.new('1').image(image.extract((2, 0)))
    s.new('2').image(image.extract((3, 0)))
    i = ITEM.from_structure(s['0'], name=prefix + 'bookshelf', extract_offset=(0, 16)) \
            .display_name(disp_prefix + 'Bookshelves')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input(material, amount)

def do_bureau(image, variant=None):
    prefix, disp_prefix, material, amount = unpack_variant(variant, 'wood', 20)

    s = STRUCTURE.new(prefix + 'bureau') \
            .mesh(BUREAU_MESH) \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1) \
            .image(image.extract((0, 0), (1, 2)))
    i = ITEM.from_structure(s).display_name(disp_prefix + 'Bureau')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input(material, amount)

    s = STRUCTURE.new(prefix + 'bureau/wide') \
            .mesh(WIDE_BUREAU_MESH) \
            .shape(structure.solid(2, 1, 1)) \
            .layer(1) \
            .image(image.extract((1, 0), (2, 2)))
    i = ITEM.from_structure(s).display_name('Wide ' + disp_prefix + 'Bureau')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input(material, amount)

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
            .shape(structure.Shape(1, 1, 1, [B_SOLID_SHAPE(S_RAMP_N) | B_OCCUPIED])) \
            .layer(1) \
            .image(image)
    i = ITEM.from_structure(s, name='stair').display_name('Stairs')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input('wood', 10)

TORCH_VARIANTS = (
        ('red', 'Red', (255, 32, 32)),
        ('orange', 'Orange', (255, 130, 32)),
        ('yellow', 'Yellow', (255, 255, 32)),
        ('green', 'Green', (32, 255, 32)),
        ('blue', 'Blue', (32, 64, 255)),
        ('purple', 'Purple', (200, 32, 255)),
        )

def do_torch(image):
    anim = Anim([image.extract((i, 0), (1, 2)) for i in range(4)], 4)
    s = STRUCTURE.new('torch') \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1) \
            .mesh_part(meshes.top(1, 1, 1), anim) \
            .mesh_part(meshes.front(1, 1, 1), anim.still()) \
            .light((16, 16, 32), (255, 230, 200), 300)
    i = ITEM.from_structure(s).display_name('Torch')
    r = RECIPE.from_item(i) \
            .station('workbench') \
            .input('wood', 2) \
            .input('stone', 1)

def do_torch_variant(image, idx, v):
    color, disp_base, light_color = v

    anim = Anim([image.extract((i, 0), (1, 2)) for i in range(4)], 4)
    s = STRUCTURE.new('torch/' + color) \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1) \
            .mesh_part(meshes.top(1, 1, 1), anim) \
            .mesh_part(meshes.front(1, 1, 1), anim.still()) \
            .light((16, 16, 32), light_color, 300)
    i = ITEM.from_structure(s).display_name(disp_base + ' Torch')
    r = RECIPE.new('torch/%d/%s' % (idx, color)) \
            .display_name(disp_base + ' Torch') \
            .station('workbench') \
            .ability('blueprint/colored_torches') \
            .input('torch', 10) \
            .input('gem/' + color, 1) \
            .output('torch/' + color, 10)


def init():
    tiles = loader('tiles', unit=TILE_SIZE)
    icons = loader('icons', unit=TILE_SIZE)
    structures = loader('structures', unit=TILE_SIZE)

    # Bed and table

    furniture = structures('furniture.png')

    v_iron = ('iron', 'Iron', 'bar/iron', 10)
    do_bed(furniture.extract((0, 0), (2, 3)))
    do_bed(furniture.extract((12, 0), (2, 3)), ('double', 'Double ', 'wood', 30))
    do_table(furniture.extract((2, 0), (2, 3)))
    do_table(furniture.extract((2, 3), (2, 3)), v_iron)
    do_bureau(furniture.extract((8, 0), (3, 2)))
    do_bureau(furniture.extract((8, 3), (3, 2)), v_iron)
    do_statue(structures('statue.png'))
    do_shelf_collider()
    do_shelves(furniture.extract((4, 0), (4, 3)))
    do_shelves(furniture.extract((4, 3), (4, 3)), v_iron)
    do_trophy(structures('trophy.png'))
    do_fountain(structures('fountain.png'))
    do_stair(structures('stair.png'))

    ITEM.new('book') \
            .display_name('Book') \
            .icon(icons('gervais_roguelike/AngbandTk_book.png'))

    do_torch(structures('torch.png'))
    for i, v in enumerate(TORCH_VARIANTS):
        do_torch_variant(structures('torch-%s.png' % v[0]), i, v)
