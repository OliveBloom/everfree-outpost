from outpost_server.core import timer, use
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import tool

def register(basename, tool_name=None):
    t_closed = DATA.template(basename + '/closed')
    t_opening = DATA.template(basename + '/opening')
    t_open = DATA.template(basename + '/open')
    t_closing = DATA.template(basename + '/closing')

    @use.structure(t_closed)
    def use_closed(e, s, args):
        s.replace(t_opening)
        timer.schedule_obj(s, s.engine.now() + 250, timeout_opening)

    def timeout_opening(s):
        s.replace(t_open)
        timer.schedule_obj(s, s.engine.now() + 3000, timeout_open)

    def timeout_open(s):
        s.replace(t_closing)
        timer.schedule_obj(s, s.engine.now() + 250, timeout_closing)

    def timeout_closing(s):
        s.replace(t_closed)

    if tool_name is not None:
        add_tool_handlers(basename, tool_name)

def add_tool_handlers(basename, tool_name):
    """Make door variants all behave like the `closed` variant when hit with a tool."""
    t_closed = DATA.template(basename + '/closed')
    t_opening = DATA.template(basename + '/opening')
    t_open = DATA.template(basename + '/open')
    t_closing = DATA.template(basename + '/closing')

    @tool.handler(tool_name, t_opening)
    def tool_opening(e, s, args):
        tool.call_handler(tool_name, t_closed, e, s, args)

    @tool.handler(tool_name, t_open)
    def tool_open(e, s, args):
        tool.call_handler(tool_name, t_closed, e, s, args)

    @tool.handler(tool_name, t_closing)
    def tool_closing(e, s, args):
        tool.call_handler(tool_name, t_closed, e, s, args)
