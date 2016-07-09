import shlex

from outpost_server.core import chat, engine, types, util
from outpost_server.core.data import DATA
from outpost_server.core.eval import eval_or_exec
from outpost_server.core.types import V3
from outpost_server.core.engine import StablePlaneId
from outpost_server.outpost.lib import appearance, util as util2
from outpost_server.outpost.lib.consts import *

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
    util2.forest_check(pawn)

    pos = pawn.pos()
    pawn.extra()['home_pos'] = pos

    client.send_message('Set home to %d, %d, %d' % (pos.x, pos.y, pos.z))

@chat.command(sethome.doc)
def home(client, args):
    pawn = client.pawn()
    util2.forest_check(pawn)

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

# Special: /l is handled in Rust code

@chat.command('/l <message>: Send a chat message to nearby players only')
def l(client, args):
    raise RuntimeError('/l should be handled in rust')


# Superuser commands

def parse_player_name(client, name):
    if name is None:
        return client
    c = client.engine.client_by_name(name)
    if c is None:
        raise ValueError('No such player: %r' % name)
    return c

def parse_item_name(name):
    item = DATA.get_item(name)
    if item is None:
        raise ValueError('No such item: %r' % name)
    return item

def parse_template_name(name):
    template = DATA.get_template(name)
    if template is None:
        raise ValueError('No such template: %r' % name)
    return template

def parse_coords(s):
    xyz = s.split(',')
    if len(xyz) != 3:
        raise ValueError('Expected 3 coordinates, got %d (%r)' % (len(xyz), s))
    x, y, z = xyz
    return V3(int(x), int(y), int(z))

@chat.su_command('''
        /tp [<who>] <where>: Teleport someone (default: you) to a location
        Options for <where>:
        - `[<plane_id>:]<x>,<y>,<z>`: absolute location, optionally on a specific plane
        - `+<x>,<y>,<z>`: offset from the current location
        - `<player_name>`: a player's current location
        Player names containing spaces should be quoted.
''')
def tp(client, args):
    args = shlex.split(args)
    eng = client.engine

    def parse_args(args):
        if len(args) == 1:
            return client.pawn(), args[0]
        elif len(args) == 2:
            who = client.engine.client_by_name(args[0])
            if who is None:
                raise ValueError('No such player: %r' % args[0])
            return who.pawn(), args[1]
        else:
            raise ValueError('Expected 1 or 2 arguments, got %d' % len(args))

    try:
        e, dest_str = parse_args(args)
        
        plane_id = None
        dest_client = client.engine.client_by_name(dest_str)
        if dest_client is not None:
            dest = dest_client.pawn().pos()
            plane_id = dest_client.pawn().plane().stable_id()
        elif dest_str.startswith('+'):
            dest = e.pos() + parse_coords(dest_str[1:])
        elif ':' in dest_str:
            plane_str, _, pos_str = dest_str.partition(':')
            plane_id = StablePlaneId(int(plane_str))
            dest = parse_coords(pos_str)
        else:
            dest = parse_coords(dest_str)

        if plane_id is None:
            e.teleport(dest)
            client.send_message('Teleported %r to %s' % (e.controller().name(), dest))
        else:
            e.teleport_plane(plane_id, dest)
            client.send_message('Teleported %r to %s on plane %d' %
                    (e.controller().name(), dest, plane_id.raw))

    except Exception as e:
        client.send_message('Error: %r' % e)

@chat.su_command("/give [<who>] <item> [<count>]: Add items to a player's inventory")
def give(client, args):
    def parse_args(args):
        if len(args) == 1:
            return None, args[0], 1
        elif len(args) == 2:
            if args[1].isdigit():
                return None, args[0], args[1]
            else:
                return args[0], args[1], None
        elif len(args) == 3:
            return args
        else:
            raise ValueError('Expected 1, 2, or 3 arguments, got %d' % len(args))

    try:
        who_str, item_str, count_str = parse_args(shlex.split(args))

        who = parse_player_name(client, who_str)
        item = parse_item_name(item_str)
        count = int(count_str)

        actual_count = who.pawn().inv('main').bulk_add(item, count)
        client.send_message('Gave %d %s to %r' % (actual_count, item.name, who.name()))

    except Exception as e:
        client.send_message('Error: %r' % e)

@chat.su_command('/place <structure>: Place a structure at your current location')
def place(client, args):
    try:
        template = parse_template_name(args)
        e = client.pawn()
        pos = util.hit_tile(e)
        e.plane().create_structure(pos, template)

        client.send_message('Placed %s at %s' % (template.name, pos))

    except Exception as e:
        client.send_message('Error: %r' % e)

@chat.su_command('/destroy: Destroy a structure at your current location')
def destroy(client, args):
    try:
        s = util.hit_structure(client.pawn())
        if s is None:
            raise ValueError('No structure at that location')
        template = s.template()
        pos = s.pos()
        s.destroy()

        client.send_message('Destroyed %s at %s' % (template.name, pos))

    except Exception as e:
        client.send_message('Error: %r' % e)

@chat.su_command('/tribe <E|P|U|A>: Change the tribe of your character')
def tribe(client, args):
    try:
        args = args.strip()
        if args not in 'EPUA':
            raise ValueError('Expected one of E, P, U, A; got %r' % args)

        e = client.pawn()
        appearance.set_tribe(e, args)
        client.send_message('Set tribe to %s' % args)

    except Exception as e:
        client.send_message('Error: %r' % e)

@chat.su_command('/kick <name> <msg>: Kick a player from the server')
def kick(client, args):
    try:
        args = shlex.split(args)
        name = args[0]
        msg = ' '.join(args[1:]) or 'Kicked'
        c = client.engine.client_by_name(name)
        assert c is not None, 'no such client: %r' % name
        c.kick(msg)

    except Exception as e:
        client.send_message('Error: %r' % e)


class FunctionObject:
    def __init__(self, obj, f):
        self.obj = obj
        self.f = f

    def __getattr__(self, k):
        return getattr(self.obj, k)

    def __repr__(self):
        return repr(self.obj)

    def __str__(self):
        return str(self.obj)

    def __call__(self, *args, **kwargs):
        return self.f(*args, **kwargs)

def _build_eval_globals():
    ctx = {
            'DATA': DATA,
            'util': util,
            }
    ctx.update(engine.__dict__)
    ctx.update(types.__dict__)
    return ctx
EVAL_GLOBALS = _build_eval_globals()

@chat.su_command('/eval <code>: Evaluate a Python expression or statement', name='eval')
def eval_(client, args):
    eng = client.engine
    c = client
    e = client.pawn()
    p = e.plane()
    s = util.hit_structure(e)
    b = p.get_block(util.hit_tile(e))
    dct = {
            'eng': eng,
            'c': FunctionObject(c, lambda n: eng.client_by_name(n)),
            'e': FunctionObject(e, lambda n: eng.client_by_name(n).pawn()),
            'p': p,
            's': s,
            'b': b,
            }

    try:
        result = eval_or_exec(args, EVAL_GLOBALS, dct)
        client.send_message('Result: %r' % (result,))
    except Exception as e:
        client.send_message('Error: %r' % (e,))
