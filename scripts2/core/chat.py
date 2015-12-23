from outpost_server.core.engine import ClientProxy

class Handler:
    """Chat command handler.  This is a wrapped function that also contains
    some metadata about the command."""

    def __init__(self, func, doc=None):
        self.func = func
        self.doc = doc

    def __call__(self, client, args):
        self.func(client, args)


_HANDLERS = {}

def client_chat_command(eng, cid, line):
    """Handler for the `client_chat_command` script hook."""
    client = ClientProxy(eng, cid)

    cmd, _, args = line.partition(' ')
    assert cmd[0] == '/'
    cmd = cmd[1:]

    handler = _HANDLERS.get(cmd)
    if handler is not None:
        handler.func(client, args)
    else:
        eng.script_cb_chat_command(cid, line)
        #client.send_message('Unknown command: /%s' % cmd)

def register_command(cmd, handler):
    """Add a Handler to _HANDLERS, checking for duplicate entries."""
    assert cmd not in _HANDLERS, \
            'duplicate registration for chat command %r' % cmd
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
        register_command(f.__name__, h)
        return h
    return decorator

@command('/help <command>: Show detailed info about <command>')
def help(client, args):
    cmd = args.strip()
    if cmd == '':
        client.send_message('Commands: %s' % (', '.join(sorted(_HANDLERS.keys()))))
        client.send_message('Use "/help <command>" for more information')
        return
    if cmd.startswith('/'):
        cmd = cmd[1:]
    if cmd not in _HANDLERS:
        client.send_message('Unknown command: /%s' % cmd)
        return

    doc = _HANDLERS[cmd].doc
    if doc is None:
        client.send_message('No information is available on /%s' % cmd)
        return

    for line in doc.strip().splitlines():
        client.send_message(line.strip())

def init(hooks):
    hooks.client_chat_command(client_chat_command)
