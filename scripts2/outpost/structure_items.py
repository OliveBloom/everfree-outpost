from outpost_server.outpost.lib import mallet, structure_items

structure_items.register('road', 'road/center/v0')
mallet.register('road/', mallet.TERRAIN_VARIANTS)

structure_items.register('house_floor', 'wood_floor/center/v0')
mallet.register('wood_floor/', mallet.TERRAIN_VARIANTS)
