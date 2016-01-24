from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import load


def init():
    tools = load('icons/tools.png', unit=ICON_SIZE)

    pick = ITEM.new('pick').display_name('Pickaxe').icon(tools.extract((1, 0)))
    RECIPE.new('pick') \
            .display_name('Pickaxe') \
            .station('anvil') \
            .inputs({'wood': 10, 'stone': 10}) \
            .output('pick', 5)

    axe = ITEM.new('axe').display_name('Axe').icon(tools.extract((3, 0)))
    RECIPE.from_item(axe) \
            .station('anvil') \
            .inputs({'wood': 10, 'stone': 10})

    shovel = ITEM.new('shovel').display_name('Shovel').icon(tools.extract((0, 0)))
    RECIPE.from_item(shovel) \
            .station('anvil') \
            .inputs({'wood': 10, 'stone': 10})

    mallet = ITEM.new('mallet').display_name('Mallet').icon(tools.extract((2, 0)))
    RECIPE.from_item(mallet) \
            .station('anvil') \
            .input('wood', 20)
