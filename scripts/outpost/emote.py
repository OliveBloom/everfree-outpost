from outpost_server.core import chat
from outpost_server.core.data import DATA

# TODO: move this into a library somewhere
def facing_to_dir(facing):
    idx = 3 * (facing.x + 1) + (facing.y + 1)
    return [5, 4, 3, 6, 0, 2, 7, 0, 1][idx]

@chat.command()
def sit(client, args):
    e = client.pawn()
    dir_ = facing_to_dir(e.facing())
    e.set_anim(DATA.animation_id('pony//sit-%d' % dir_))

@chat.command()
def sleep(client, args):
    e = client.pawn()
    dir_ = facing_to_dir(e.facing())
    if dir_ in range(3, 6):
        e.set_anim(DATA.animation_id('pony//sleep-4'))
    else:
        e.set_anim(DATA.animation_id('pony//sleep-0'))
