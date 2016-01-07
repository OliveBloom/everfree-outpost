from outpost_server.core.engine import ClientProxy

class Handler:
    """Chat command handler.  This is a wrapped function that also contains
    some metadata about the command."""

    def __init__(self, func, doc=None, name=None, need_su=False):
        self.func = func
        self.doc = doc
        self.name = name or func.__name__
        self.need_su = need_su

    def __call__(self, client, args):
        self.func(client, args)


_HANDLERS = {}
_SU_HANDLERS = {}

def get_handler(cmd, client):
    h = _HANDLERS.get(cmd)
    if h is None and client.is_superuser():
        h = _SU_HANDLERS.get(cmd)
    return h

def list_commands(client):
    cmds = list(_HANDLERS.keys())
    if client.is_superuser():
        cmds.extend(_SU_HANDLERS.keys())
    cmds.sort()
    return cmds

def client_chat_command(eng, cid, line):
    """Handler for the `client_chat_command` script hook."""
    client = ClientProxy(eng, cid)

    cmd, _, args = line.partition(' ')
    assert cmd[0] == '/'
    cmd = cmd[1:]

    handler = get_handler(cmd, client)
    if handler is not None:
        handler.func(client, args)
    else:
        client.send_message('Unknown command: /%s' % cmd)

def register_command(handler):
    """Add a Handler to _HANDLERS, checking for duplicate entries."""
    cmd = handler.name
    assert cmd not in _HANDLERS and cmd not in _SU_HANDLERS, \
            'duplicate registration for chat command %r' % cmd
    if handler.need_su:
        _SU_HANDLERS[cmd] = handler
    else:
        _HANDLERS[cmd] = handler

def command(*args, **kwargs):
    """Decorator for registering chat command handlers.  Arguments are passed
    through to the `Handler` constructor.

    Usage:
        @command                            # Default options
        @command(doc='/cmd: Blah blah')     # Set documentation for /help
    """

    # Support no-argument usage
    if len(args) == 1 and len(kwargs) == 0 and hasattr(args[0], '__call__'):
        return command()(*args)

    def decorator(f):
        h = Handler(f, *args, **kwargs)
        register_command(h)
        return h
    return decorator

def su_command(*args, **kwargs):
    return command(*args, need_su=True, **kwargs)

@command('/help <command>: Show detailed info about <command>')
def help(client, args):
    cmd = args.strip()
    if cmd == '':
        client.send_message('Commands: %s' % (', '.join(list_commands(client))))
        client.send_message('Use "/help <command>" for more information')
        return

    if cmd.startswith('/'):
        cmd = cmd[1:]
    handler = get_handler(cmd, client)
    if handler is None:
        client.send_message('Unknown command: /%s' % cmd)
        return

    doc = handler.doc
    if doc is None:
        client.send_message('No information is available on /%s' % cmd)
        return

    for line in doc.strip().splitlines():
        client.send_message(line.strip())

def init(hooks):
    hooks.client_chat_command(client_chat_command)
