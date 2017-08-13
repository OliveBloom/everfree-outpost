from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.geom import Mesh
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes


SOLID_ATTACH_MESH = Mesh(
        meshes.quad_y(32,  0, 32, 16, 48) +
        meshes.quad_z(48,  0, 32,  0, 32))
SOLID_ATTACH_BOUNDS = ((0, 0, 16), (32, 32, 48))

FRONT_ATTACH_MESH = Mesh(
        meshes.quad_y(32,  0, 32, 16, 48))
FRONT_ATTACH_BOUNDS = ((0, 32, 16), (32, 32, 48))

BOTTOM_ATTACH_MESH = Mesh(
        meshes.quad_z(16,  0, 32,  0, 32))
BOTTOM_ATTACH_BOUNDS = ((0, 0, 16), (32, 32, 16))


def init():
    structures = loader('structures', unit=TILE_SIZE)

    img_attach = structures('furniture-attachments.png')

    s = STRUCTURE.new('teleporter/attached') \
            .shape(structure.empty(1, 1, 1)) \
            .mesh(FRONT_ATTACH_MESH) \
            .image_bounds(FRONT_ATTACH_BOUNDS) \
            .image(structures('crystal-formation.png')) \
            .layer(2) \
            .light((16, 16, 32), (48, 48, 96), 50)

    s = STRUCTURE.new('lamp/attached') \
            .shape(structure.empty(1, 1, 1)) \
            .mesh(SOLID_ATTACH_MESH) \
            .image_bounds(SOLID_ATTACH_BOUNDS) \
            .image(img_attach.extract((0, 0), (1, 2))) \
            .layer(2) \
            .light((16, 16, 48), (255, 230, 200), 300)
    s = STRUCTURE.new('lamp/off/attached') \
            .shape(structure.empty(1, 1, 1)) \
            .mesh(SOLID_ATTACH_MESH) \
            .image_bounds(SOLID_ATTACH_BOUNDS) \
            .image(img_attach.extract((1, 0), (1, 2))) \
            .layer(2)

    s = STRUCTURE.new('workbench/attached') \
            .shape(structure.empty(1, 1, 1)) \
            .mesh(BOTTOM_ATTACH_MESH) \
            .image_bounds(BOTTOM_ATTACH_BOUNDS) \
            .image(img_attach.extract((2, 0))) \
            .layer(2)

