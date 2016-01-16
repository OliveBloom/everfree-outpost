from outpost_server.core import use
from outpost_server.core.types import V3
from outpost_server.outpost.lib.consts import *

ENTRANCE_POS = V3(128, 128, 12) * TILE_SIZE

@use.structure('dungeon_entrance')
def use_entrance(e, s, args):
    pid = s.extra().get('plane')
    if pid is None:
        p = s.engine.create_plane('Dungeon')
        p.extra()['dest'] = e.pos()
        pid = p.stable_id()
        s.extra()['plane'] = pid

    e.teleport_plane(pid, ENTRANCE_POS)

@use.structure('dungeon_exit')
def use_exit(e, s, args):
    dest = s.plane().extra().get('dest', SPAWN_POINT)
    e.teleport_plane(STABLE_PLANE_FOREST, dest)

