from outpost_server.core import chat
from outpost_server.core import V3

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
            (plane.name, plane.stable_id(), pos.x, pos.y, pos.z)
    client.send_message(msg)

@chat.command('/spawn: Teleport to the spawn point')
def spawn(client, args):
    # TODO: constants for PLANE_FOREST and spawn point
    client.pawn().teleport_plane(2, V3(32, 32, 0))

# Client-side commands (included here for /help purposes only)

@chat.command('/ignore <name>: Hide chat messages from named player')
def ignore(client, args):
    raise RuntimeError('/ignore should be handled client-side')

@chat.command('/unignore <name>: Stop hiding chat messages from <name>')
def unignore(client, args):
    raise RuntimeError('/unignore should be handled client-side')

# Lua commands (included here for /help purposes)

@chat.command('/permit <name>: Give <name> permission to bypass your ward')
def permit(client, args):
    client._eng.script_cb_chat_command(client.id, '/permit ' + args)

@chat.command("/revoke <name>: Revoke <name>'s permission to bypass your ward")
def revoke(client, args):
    client._eng.script_cb_chat_command(client.id, '/revoke ' + args)
