from outpost_server.core import chat
from outpost_server.core.data import DATA

# TODO: move this into a library somewhere
def facing_to_dir(facing):
    idx = 3 * (facing.x + 1) + (facing.y + 1)
    return [2, 2, 2, 3, 0, 1, 0, 0, 0][idx]

@chat.command()
def sit(client, args):
    e = client.pawn()
    dir_ = facing_to_dir(e.facing())
    e.set_anim(DATA.animation_id('pony//sit-%d' % dir_))

@chat.command()
def sleep(client, args):
    e = client.pawn()
    dir_ = facing_to_dir(e.facing())
    if dir_ == 2:
        e.set_anim(DATA.animation_id('pony//sleep-2'))
    else:
        e.set_anim(DATA.animation_id('pony//sleep-0'))
