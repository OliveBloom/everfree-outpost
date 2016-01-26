from outpost_data.core import image2
from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes


OPEN_DOOR_SHAPE = structure.Shape(3, 1, 2, (
    'solid', 'floor', 'solid',
    'solid', 'empty', 'solid',
    ))

def do_dungeon_door(basename, img, door_anim):
    sb = STRUCTURE.prefixed(basename) \
            .shape(structure.solid(3, 1, 2)) \
            .layer(1)

    door_model = structure.Model2(meshes.front(3, 1, 2),
            ((0, 1 * TILE_SIZE, 0), (3 * TILE_SIZE, 1 * TILE_SIZE, 2 * TILE_SIZE)))

    sb.new('open') \
            .shape(OPEN_DOOR_SHAPE) \
            .part(door_model, door_anim.get_frame(-1))
    sb.new('closed').part(door_model, door_anim.get_frame(0))
    sb.new('opening').part(door_model, door_anim)
    sb.new('closing').part(door_model, door_anim.reversed())

    # The doorway needs to be drawn over top of the animated door.
    sb.mesh_part(meshes.solid(3, 1, 2), img)


def init():
    structures = loader('structures', unit=TILE_SIZE)

    door_anim_sheet = structures('cave-door.png').with_unit((3 * TILE_SIZE, 2 * TILE_SIZE))
    door_anim = Anim(
            [door_anim_sheet.extract((0, i)) for i in range(door_anim_sheet.size[1])],
            16, oneshot=True)
    do_dungeon_door('dungeon/door/key', structures('cave-doorway-keyhole.png'), door_anim)
    do_dungeon_door('dungeon/door/puzzle', structures('cave-doorway-plain.png'), door_anim)
