from outpost_server.outpost.lib import autorotate, door, mallet, structure_items

autorotate.register_floor('road')
mallet.register('road/', mallet.TERRAIN_VARIANTS)

autorotate.register_floor('house_floor', 'wood_floor')
mallet.register('wood_floor/', mallet.TERRAIN_VARIANTS)

def wall_and_door(name, tool, extra_variants=()):
    autorotate.register_wall('%s_wall' % name)
    mallet.register('%s_wall/' % name,
            ('edge/horiz',) + extra_variants + mallet.COMMON_WALL_VARIANTS)
    structure_items.register('%s_door' % name, '%s_wall/door/closed' % name, tool)
    door.register('%s_wall/door' % name, tool_name=tool)

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
