from outpost_server.outpost.lib import autorotate, door, mallet, structure_items

autorotate.register_floor('road')
mallet.register('road/', mallet.TERRAIN_VARIANTS)

autorotate.register_floor('house_floor', 'wood_floor')
mallet.register('wood_floor/', mallet.TERRAIN_VARIANTS)


autorotate.register_fence('fence', 'fence_post')
mallet.register('fence/', mallet.WALL_VARIANTS)


structure_items.register('bed')
structure_items.register('table')
structure_items.register('trophy')
structure_items.register('fountain')
structure_items.register('torch')
structure_items.register('stair', 'stair/n')
structure_items.register('wood_pillar', 'pillar/wood')
structure_items.register('stone_pillar', 'pillar/stone')
structure_items.register('statue', 'statue/e')
mallet.register('statue/', ('e', 's', 'w', 'n'))


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

# ruined_wall doesn't have a proper door, so we can't use wall_and_door
autorotate.register_wall('ruined_wall')
mallet.register('ruined_wall/',
        ('edge/horiz', 'window/v0', 'window/v1',) + mallet.WALL_VARIANTS)
structure_items.register('ruined_door', 'ruined_wall/door/open', 'pickaxe')
