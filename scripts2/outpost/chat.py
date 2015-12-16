from outpost_server.core import chat

@chat.command
def count(client, args):
    count = client.engine.num_clients()
    msg = '%d player%s online' % (count, 's' if count != 1 else '')
    client.send_message(msg)
