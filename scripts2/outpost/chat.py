from outpost_server.core import chat

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
