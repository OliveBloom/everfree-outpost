from .block import BlockBuilder
from .structure import StructureBuilder
from .item import ItemBuilder
from .recipe import RecipeBuilder
from .sprite import SpriteBuilder
from .loot_table import LootTableBuilder
from .extra import ExtraBuilder

__all__ = (
        'INSTANCES',
        'BLOCK', 'STRUCTURE', 'ITEM', 'RECIPE', 'SPRITE', 'LOOT_TABLE',
        'EXTRA',
        )


INSTANCES = dict(
        block = BlockBuilder(),
        structure = StructureBuilder(),
        item = ItemBuilder(),
        recipe = RecipeBuilder(),
        sprite = SpriteBuilder(),
        loot_table = LootTableBuilder(),
        extra = ExtraBuilder(),
        )

BLOCK = INSTANCES['block']
STRUCTURE = INSTANCES['structure']
ITEM = INSTANCES['item']
RECIPE = INSTANCES['recipe']
SPRITE = INSTANCES['sprite']
LOOT_TABLE = INSTANCES['loot_table']
EXTRA = INSTANCES['extra']
