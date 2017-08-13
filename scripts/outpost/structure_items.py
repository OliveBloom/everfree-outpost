from outpost_server.core import use, util
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import autorotate, door, mallet, structure_items, tool, ward

autorotate.register_floor('road')
mallet.register('road/', mallet.TERRAIN_VARIANTS)

autorotate.register_floor('house_floor', 'wood_floor')
mallet.register('wood_floor/', mallet.TERRAIN_VARIANTS)

for color in ('red', 'orange', 'yellow', 'green', 'blue', 'purple'):
    autorotate.register_floor('wood_floor/' + color)
    mallet.register('wood_floor/%s/' % color, mallet.TERRAIN_VARIANTS)




autorotate.register_fence('fence', 'fence_post')
mallet.register('fence/', mallet.WALL_VARIANTS)
mallet.register('fence/', ('end/fancy/e', 'end/fancy/w'))
structure_items.register('fence_gate', 'fence/gate/closed', tool='axe')
door.register_use('fence/gate', tool_name='axe')


structure_items.register('bed')
structure_items.register('double/bed')
structure_items.register('trophy')
structure_items.register('fountain')
structure_items.register('stair', 'stair/n')
structure_items.register('wood_pillar', 'pillar/wood')
structure_items.register('stone_pillar', 'pillar/stone')
structure_items.register('statue', 'statue/e')
mallet.register('statue/', ('e', 's', 'w', 'n'))

structure_items.register('table')
structure_items.register_base('table', 'table')
structure_items.register('iron/table')
structure_items.register_base('iron/table', 'table')

structure_items.register('torch')
for color in ('red', 'orange', 'yellow', 'green', 'blue', 'purple'):
    structure_items.register('torch/' + color)



LAMP_ITEM = DATA.item('lamp')
LAMP = DATA.template('lamp')
LAMP_OFF = DATA.template('lamp/off')
LAMP_ATTACHED = DATA.template('lamp/attached')
LAMP_OFF_ATTACHED = DATA.template('lamp/off/attached')

LAMP_TOGGLE = {
        LAMP: LAMP_OFF,
        LAMP_OFF: LAMP,
        LAMP_ATTACHED: LAMP_OFF_ATTACHED,
        LAMP_OFF_ATTACHED: LAMP_ATTACHED,
        }

@use.structure(LAMP)
@use.structure(LAMP_OFF)
@use.structure(LAMP_ATTACHED)
@use.structure(LAMP_OFF_ATTACHED)
def use_lamp(e, s, args):
    ward.check(e, s.pos())
    s.replace(LAMP_TOGGLE[s.template()])

@tool.axe(LAMP)
@tool.axe(LAMP_OFF)
@tool.axe(LAMP_ATTACHED)
@tool.axe(LAMP_OFF_ATTACHED)
def axe_lamp(e, s, args):
    structure_items.take(e, s, LAMP_ITEM)

@use.item(LAMP_ITEM)
def place_lamp(e, args):
    if structure_items.check_attachment(LAMP_ATTACHED, e.plane(), util.hit_tile(e)):
        structure_items.place(e, LAMP_ITEM, LAMP_ATTACHED)
    else:
        structure_items.place(e, LAMP_ITEM, LAMP)

structure_items.register_attachment(LAMP_ATTACHED, 'table')
structure_items.register_attachment(LAMP_OFF_ATTACHED, 'table')



def wall_and_door(name, tool, extra_variants=()):
    horiz_variants = ('edge/horiz',) + extra_variants

    autorotate.register_wall('%s_wall' % name)
    mallet.register('%s_wall/' % name,
            horiz_variants + mallet.COMMON_WALL_VARIANTS)
    structure_items.register('%s_door' % name, '%s_wall/door/closed' % name, tool)
    door.register_use('%s_wall/door' % name, tool_name=tool)

    for v in horiz_variants:
        structure_items.register_base('%s_wall/%s' % (name, v), 'wall/horiz')
    structure_items.register_base('%s_wall/tee/n' % name, 'wall/horiz')

wall_and_door('interior', 'axe')
wall_and_door('brick', 'pickaxe')
wall_and_door('wood', 'axe', ('window/v0',))
wall_and_door('stone', 'pickaxe', ('window/v0', 'window/v1',))
wall_and_door('cottage', 'axe',
        ('variant/v0', 'variant/v1', 'variant/v2', 'window/v0', 'window/v1',))
wall_and_door('iron', 'pickaxe')

# ruined_wall doesn't have a proper door, so we can't use wall_and_door
autorotate.register_wall('ruined_wall')
mallet.register('ruined_wall/',
        ('edge/horiz', 'window/v0', 'window/v1',) + mallet.WALL_VARIANTS)
structure_items.register('ruined_door', 'ruined_wall/door/open', 'pickaxe')
