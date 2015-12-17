from outpost_server.core import chat
from outpost_server.core import V3

@chat.command
def count(client, args):
    count = client.engine.num_clients()
    msg = '%d player%s online' % (count, 's' if count != 1 else '')
    client.send_message(msg)

@chat.command
def where(client, args):
    pawn = client.pawn()
    plane = pawn.plane()
    pos = pawn.pos()
    msg = 'Location: %s (%d), %d, %d, %d' % \
            (plane.name, plane.stable_id(), pos.x, pos.y, pos.z)
    client.send_message(msg)

@chat.command
def spawn(client, args):
    # TODO: constants for PLANE_FOREST and spawn point
    client.pawn().teleport_plane(2, V3(32, 32, 0))
