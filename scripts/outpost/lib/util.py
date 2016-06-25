from outpost_server.outpost.lib.consts import *

def forest_check(e):
    if e.plane().stable_id() != STABLE_PLANE_FOREST:
        e.controller().send_message("That doesn't work here.")
        raise RuntimeError('tried to perform forest-only action outside the forest')

def facing_to_dir(facing):
    idx = 3 * (facing.x + 1) + (facing.y + 1)
    return [2, 2, 2, 3, 0, 1, 0, 0, 0][idx]
