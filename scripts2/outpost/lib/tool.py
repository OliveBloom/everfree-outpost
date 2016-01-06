from outpost_server.core import alias, util
from outpost_server.core.data import DATA

_HANDLERS = {}

def handler(tool_name, template):
    template = alias.template(template)
    def register(f):
        _HANDLERS.setdefault(tool_name, {})[template] = f
        return f
    return register

def call_handler(tool_name, template, e, s, args):
    tool_handlers = _HANDLERS.get(tool_name)
    if tool_handlers is None:
        return

    handler = tool_handlers.get(alias.template(template))
    if handler is not None:
        handler(e, s, args)

def use(e, tool_name, args=None):
    s = util.hit_structure(e)
    if s is None:
        return

    tool_handlers = _HANDLERS.get(tool_name)
    if tool_handlers is None:
        return

    handler = tool_handlers.get(alias.template(s.template()))
    if handler is not None:
        handler(e, s, args)
    else:
        # TODO: hack to allow pass through to lua for missing handlers - remove
        return False

pickaxe = lambda t: handler('pickaxe', t)
axe = lambda t: handler('axe', t)
