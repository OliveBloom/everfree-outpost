from outpost_server.outpost.lib import autorotate, door, mallet, structure_items

autorotate.register_floor('road')
mallet.register('road/', mallet.TERRAIN_VARIANTS)

autorotate.register_floor('house_floor', 'wood_floor')
mallet.register('wood_floor/', mallet.TERRAIN_VARIANTS)

autorotate.register_wall('interior_wall')
mallet.register('interior_wall/', mallet.WALL_VARIANTS)
structure_items.register('interior_door', 'interior_wall/door/closed', 'axe')
door.register('interior_wall/door', tool_name='axe')
