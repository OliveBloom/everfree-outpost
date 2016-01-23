from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.geom import Mesh
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes

def do_teleporter(image):
    s = STRUCTURE.new('teleporter') \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.front(1, 1, 1)) \
            .image(image) \
            .layer(1) \
            .light((16, 16, 16), (48, 48, 96), 50)
    i = ITEM.from_structure(s).display_name('Teleporter')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('crystal', 50)

def do_dungeon_entrance(image):
    sb = STRUCTURE.child() \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.front(1, 1, 1)) \
            .image(image) \
            .layer(1) \
            .light((16, 16, 16), (96, 48, 48), 50)
    sb.new('dungeon_entrance')
    sb.new('dungeon_exit')

def do_ward(image):
    s = STRUCTURE.new('ward') \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.solid(1, 1, 1)) \
            .image(image) \
            .layer(1) \
            .light((16, 16, 32), (48, 48, 96), 50)
    i = ITEM.from_structure(s, extract_offset=(0, 10)).display_name('Ward')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 10) \
            .input('crystal', 1)

def do_sign(image):
    s = STRUCTURE.new('sign') \
            .shape(structure.solid(1, 1, 1)) \
            .mesh(meshes.solid(1, 1, 1)) \
            .image(image) \
            .layer(1)
    i = ITEM.from_structure(s).display_name('Sign')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 5)

def do_pillar(image, kind, desc):
    s = STRUCTURE.new('pillar/' + kind) \
            .shape(structure.solid(1, 1, 2)) \
            .mesh(meshes.solid(1, 1, 2)) \
            .image(image) \
            .layer(1)
    i = ITEM.from_structure(s, name=kind + '_pillar').display_name(desc + ' Pillar')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input(kind, 5)

def do_floor_structure(image, basename):
    sb = STRUCTURE.prefixed(basename) \
            .shape(structure.floor(1, 1, 1)) \
            .mesh(meshes.bottom(1, 1)) \
            .bounds(((0, 0, 0), (TILE_SIZE, TILE_SIZE, 0))) \
            .layer(0)
    for k,v in image.chop(TERRAIN_PARTS2).items():
        sb.new(k).image(v)
    return sb

def do_floor(image):
    s = do_floor_structure(image, 'wood_floor')
    i = ITEM.from_structure(s['center/v0'], name='house_floor').display_name('House Floor')
    r = RECIPE.from_item(i) \
            .station('anvil') \
            .input('wood', 5)

def do_floor_variant(image, idx, color, disp_base):
    s = do_floor_structure(image, 'wood_floor/' + color)
    i = ITEM.from_structure(s['center/v0'], name='wood_floor/' + color) \
            .display_name(disp_base + ' Floor')
    r = RECIPE.new('floor/%d/%s' % (idx, color)) \
            .display_name(disp_base + ' Floor') \
            .station('anvil') \
            .ability('blueprint/colored_floors') \
            .input('house_floor', 20) \
            .input('gem/' + color, 1) \
            .output('wood_floor/' + color, 20)

def init():
    icons = loader('icons', unit=ICON_SIZE)
    structures = loader('structures', unit=TILE_SIZE)

    ITEM.new('crystal') \
            .display_name('Crystal') \
            .icon(icons('crystal.png'))

    do_teleporter(structures('crystal-formation.png'))
    do_dungeon_entrance(structures('crystal-formation-red.png'))
    do_ward(structures('crystal-ward.png').extract((0, 0), (1, 2)))

    do_sign(structures('sign.png'))

    ITEM.new('hat') \
            .display_name('Hat') \
            .icon(icons('equip_hat_icon.png'))
    ITEM.new('party_hat') \
            .display_name('Party Hat') \
            .icon(icons('party-hat-icon.png'))
    ITEM.new('santa_hat') \
            .display_name('Santa Hat') \
            .icon(icons('santa-hat-icon.png'))

    pillars = structures('pillar.png')
    do_pillar(pillars.extract((0, 0), (1, 3)), 'wood', 'Wooden')
    do_pillar(pillars.extract((1, 0), (1, 3)), 'stone', 'Stone')

    do_floor(structures('wood-floor.png'))
    COLORS = (
            ('red', 'Red'),
            ('orange', 'Orange'),
            ('yellow', 'Yellow'),
            ('green', 'Green'),
            ('blue', 'Blue'),
            ('purple', 'Purple'),
            )
    for i, (color, desc) in enumerate(COLORS):
        do_floor_variant(structures('floor-%s.png' % color), i, color, desc)
