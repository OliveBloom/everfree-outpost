from outpost_server.core.engine import ClientProxy

_HANDLERS = {}

def client_chat_command(eng, cid, line):
    client = ClientProxy(eng, cid)

    cmd, _, args = line.partition(' ')
    assert cmd[0] == '/'
    cmd = cmd[1:]

    handler = _HANDLERS.get(cmd)
    if handler is not None:
        handler(client, args)
    else:
        eng.script_cb_chat_command(cid, line)
        #client.send_message('Unknown command: /%s' % cmd)

def register_command(cmd, handler):
    assert cmd not in _HANDLERS, \
            'duplicate registration for chat command %r' % cmd
    _HANDLERS[cmd] = handler

def command(f):
    register_command(f.__name__, f)
    return f

def init(hooks):
    hooks.client_chat_command(client_chat_command)
