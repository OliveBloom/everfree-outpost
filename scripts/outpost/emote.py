from outpost_server.core import chat
from outpost_server.core.data import DATA
from outpost_server.core.engine import Emote
from outpost_server.outpost.lib.util import facing_to_dir

@chat.command()
def sit(client, args):
    e = client.pawn()
    dir_ = facing_to_dir(e.facing())
    e.set_activity(Emote(DATA.animation_id('pony//sit-%d' % dir_)))

@chat.command()
def sleep(client, args):
    e = client.pawn()
    dir_ = facing_to_dir(e.facing())
    if dir_ == 2:
        e.set_activity(Emote(DATA.animation_id('pony//sleep-2')))
    else:
        e.set_activity(Emote(DATA.animation_id('pony//sleep-0')))
