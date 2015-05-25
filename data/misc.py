from outpost_data.builder import *
import outpost_data.images as I
from outpost_data import depthmap
from outpost_data.structure import Shape
from outpost_data.util import loader, extract

from lib.items import *
from lib.structures import *


def init(asset_path):
    tiles = loader(asset_path, 'tiles')
    structures = loader(asset_path, 'structures')

    road = mk_terrain_structures('road', structures('road.png'))
    mk_structure_item(road['road/center/v0'], 'road', 'Road') \
            .recipe('anvil', {'stone': 5}, count=2)

    anvil = mk_solid_small('anvil', structures('anvil.png'))
    mk_structure_item(anvil, 'anvil', 'Anvil') \
            .recipe('anvil', {'wood': 10, 'stone': 10})

    chest = mk_solid_small('chest', structures('chest.png'))
    mk_structure_item(chest, 'chest', 'Chest') \
            .recipe('anvil', {'wood': 20})

    teleporter = mk_solid_small('teleporter', structures('crystal-formation.png')) \
            .light((16, 16, 16), (48, 48, 96), 50)
    mk_structure_item(teleporter, 'teleporter', 'Teleporter') \
            .recipe('anvil', {'crystal': 50})

    ward = mk_solid_structure('ward', structures('crystal-ward.png'), (1, 1, 1)) \
            .light((16, 16, 32), (48, 48, 96), 50)
    mk_item('ward', 'Ward', extract(structures('crystal-ward.png'), (1, 1))) \
            .recipe('anvil', {'wood': 10, 'crystal': 1})

    mk_solid_small('dungeon_entrance', structures('crystal-formation-red.png')) \
            .light((16, 16, 16), (96, 48, 48), 50)
    mk_solid_small('dungeon_exit', structures('crystal-formation-red.png')) \
            .light((16, 16, 16), (96, 48, 48), 50)
    mk_solid_small('script_trigger', structures('crystal-formation-green.png')) \
            .light((16, 16, 16), (48, 96, 48), 50)


    mk_item('crystal', 'Crystal', extract(structures('crystal-ward.png'), (1, 0)))

    mk_item('hat', 'Hat', tiles('equip_hat_icon.png'))


