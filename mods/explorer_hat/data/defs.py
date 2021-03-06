from outpost_data.core.builder2 import *
from outpost_data.core.image2 import load
# The `pony_sprite` module contains lots of pony-specific helper functions.
from outpost_data.outpost.lib import pony_sprite

def init():
    pony = pony_sprite.get_pony_sprite()

    # Load the hat sprite (female version) from this mod's assets/ directory.
    hat_f = load('explorer-hat-f.png')
    # Call a library function to do all the had work of setting up the hat variant.
    pony_sprite.add_hat_layer('f/hat/explorer', 'f', hat_f)

    # Do the same thing for the male version.
    hat_m = load('explorer-hat-m.png')
    pony_sprite.add_hat_layer('m/hat/explorer', 'm', hat_m)

    ITEM.new('hat/explorer') \
            .display_name('Explorer Hat') \
            .icon(load('explorer-hat-icon.png'))
