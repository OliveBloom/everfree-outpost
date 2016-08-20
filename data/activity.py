from outpost_data.core import sprite, image2
from outpost_data.core.builder2 import *
from outpost_data.core.consts import *
from outpost_data.core.image2 import load, loader, Image, Anim
from outpost_data.outpost.lib.pony_sprite import *
from outpost_data.outpost.lib.sprite_util import *


_ACTIVITY_SPRITE = None
def get_activity_sprite():
    global _ACTIVITY_SPRITE
    if _ACTIVITY_SPRITE is None:
        _ACTIVITY_SPRITE = SPRITE.new('activity', (16, 16))
        _ACTIVITY_SPRITE.add_layer('default')
    return _ACTIVITY_SPRITE

def add_activity_icon(name, img):
    activity = get_activity_sprite()
    activity.add_anim(name, 1, 1)
    anim = Anim([img], 1)
    activity.add_graphics('default', name, anim)

def init():
    # Activity sprites

    bubble = SPRITE.new('activity_bubble', (32, 32))
    bubble.add_anim('default', 1, 1)
    bubble.add_layer('default')
    anim = Anim([load('sprites/misc/activity.png')], 1)
    bubble.add_graphics('default', 'default', anim)


    icons = loader('icons', unit=16)

    tools = icons('tools.png')
    add_activity_icon('none', Image((16, 16)))
    add_activity_icon('activity/kick', icons('activity-kick.png'))

