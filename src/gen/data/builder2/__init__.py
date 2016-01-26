from .block import BlockBuilder
from .structure import StructureBuilder
from .item import ItemBuilder
from .recipe import RecipeBuilder
from .loot_table import LootTableBuilder
from .extra import ExtraBuilder


INSTANCES = dict(
        block = BlockBuilder(),
        structure = StructureBuilder(),
        item = ItemBuilder(),
        recipe = RecipeBuilder(),
        loot_table = LootTableBuilder(),
        extra = ExtraBuilder(),
        )

BLOCK = INSTANCES['block']
STRUCTURE = INSTANCES['structure']
ITEM = INSTANCES['item']
RECIPE = INSTANCES['recipe']
LOOT_TABLE = INSTANCES['loot_table']
EXTRA = INSTANCES['extra']
