from outpost_server.outpost.lib import autorotate, mallet, structure_items

autorotate.register_floor('road')
mallet.register('road/', mallet.TERRAIN_VARIANTS)

autorotate.register_floor('house_floor', 'wood_floor')
mallet.register('wood_floor/', mallet.TERRAIN_VARIANTS)
