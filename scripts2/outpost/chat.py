from outpost_server.core import chat
from outpost_server.core.types import V3
from outpost_server.core.engine import StablePlaneId

# TODO: move somewhere more appropriate
STABLE_PLANE_FOREST = StablePlaneId(2)
SPAWN_POINT = V3(32, 32, 0)

@chat.command('/count: Show the number of players currently online')
def count(client, args):
    count = client.engine.num_clients()
    msg = '%d player%s online' % (count, 's' if count != 1 else '')
    client.send_message(msg)

@chat.command('/where: Show coordinates of your current position')
def where(client, args):
    pawn = client.pawn()
    plane = pawn.plane()
    pos = pawn.pos()
    msg = 'Location: %s (%d), %d, %d, %d' % \
            (plane.name(), plane.stable_id().raw, pos.x, pos.y, pos.z)
    client.send_message(msg)

@chat.command('/spawn: Teleport to the spawn point')
def spawn(client, args):
    client.pawn().teleport_plane(STABLE_PLANE_FOREST, SPAWN_POINT)

@chat.command('''
    /sethome: Set custom teleport destination
    /home: Teleport to custom destination
''')
def sethome(client, args):
    pawn = client.pawn()
    if pawn.plane().stable_id() != STABLE_PLANE_FOREST:
        client.send_message("That command doesn't work here.")
        return

    pos = pawn.pos()
    pawn.extra()['home_pos'] = pos

@chat.command(sethome.doc)
def home(client, args):
    pawn = client.pawn()
    if pawn.plane().stable_id() != STABLE_PLANE_FOREST:
        client.send_message("That command doesn't work here.")
        return

    extra = pawn.extra()
    pos = extra.get('home_pos', SPAWN_POINT)
    pawn.teleport(pos)

# Client-side commands (included here for /help purposes only)

@chat.command('/ignore <name>: Hide chat messages from named player')
def ignore(client, args):
    raise RuntimeError('/ignore should be handled client-side')

@chat.command('/unignore <name>: Stop hiding chat messages from <name>')
def unignore(client, args):
    raise RuntimeError('/unignore should be handled client-side')
