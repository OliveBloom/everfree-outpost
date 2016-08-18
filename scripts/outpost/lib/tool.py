from collections import namedtuple
from outpost_server.core import alias, util
from outpost_server.core.data import DATA
from outpost_server.outpost.lib import ward

Args = namedtuple('Args', ('kind', 'level',))

_HANDLERS = {}

def handler(tool_name, template):
    template = alias.template(template)
    def register(f):
        _HANDLERS.setdefault(tool_name, {})[template] = f
        return f
    return register

def call_handler(template, e, s, args):
    tool_handlers = _HANDLERS.get(args.kind)
    if tool_handlers is None:
        return

    handler = tool_handlers.get(alias.template(template))
    if handler is not None:
        handler(e, s, args)

def use(e, args):
    s = util.hit_structure(e)
    if s is None:
        return

    tool_handlers = _HANDLERS.get(args.kind)
    if tool_handlers is None:
        return

    handler = tool_handlers.get(alias.template(s.template()))
    if handler is not None:
        handler(e, s, args)

pickaxe = lambda t: handler('pickaxe', t)
axe = lambda t: handler('axe', t)

UI_NAMES = {
        'pickaxe': 'pick',
        'axe': 'axe',
        }


def default_check(base_delay, min_level=0):
    def default_check_impl(e, s, args):
        ward.check(e, s.pos())
        excess = args.level - min_level
        if excess < 0:
            name = UI_NAMES.get(args.kind, args.kind)
            e.controller().send_message('This %s is not strong enough.' % name)
            return None
        return base_delay * (6 - excess) // 6
    return default_check_impl

