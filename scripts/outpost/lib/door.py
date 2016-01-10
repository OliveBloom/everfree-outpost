from outpost_server.core import state_machine, timer, use
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import tool

def register(basename, delay, mid_delay=None):
    t_open = DATA.template(basename + '/open')
    t_opening = DATA.template(basename + '/opening')
    t_closed = DATA.template(basename + '/closed')
    t_closing = DATA.template(basename + '/closing')

    @state_machine.structure(t_open)
    @state_machine.structure(t_opening)
    @state_machine.structure(t_closed)
    @state_machine.structure(t_closing)
    class DoorMachine(state_machine.StateMachine):
        def init(self):
            self.state = 'closed'
            self.auto = False

        def open(self):
            if self.state in ('closed', 'closing'):
                self.cancel()
                self.auto = False
                self.transition('opening')

        def close(self):
            if self.state in ('open', 'opening'):
                self.cancel()
                self.auto = False
                self.transition('closing')

        def open_close(self):
            self.cancel()
            if self.state in ('closed', 'closing'):
                self.auto = True
                self.transition('opening')
            else:
                self.auto = False
                self.schedule(mid_delay, 'closing')

        def enter_opening(self):
            self.obj.replace(t_opening)
            self.schedule(delay, 'open')

        def enter_open(self):
            self.obj.replace(t_open)
            if self.auto:
                self.schedule(mid_delay, 'closing')
                self.auto = False

        def enter_closing(self):
            self.obj.replace(t_closing)
            self.schedule(delay, 'closed')

        def enter_closed(self):
            self.obj.replace(t_closed)

    return (t_opening, t_open, t_closing, t_closed)


def open(s):
    state_machine.get(s).open()

def close(s):
    state_machine.get(s).close()

def open_close(s):
    state_machine.get(s).open_close()


def register_use(basename, tool_name=None, delay=250, mid_delay=3000):
    t_opening, t_open, t_closing, t_closed = register(basename, delay, mid_delay)

    @use.structure(t_closed)
    def use_closed(e, s, args):
        open_close(s)

    @use.structure(t_open)
    def use_open(e, s, args):
        close(s)

    if tool_name is not None:
        # Make all variants behave the same as `t_closed`
        @tool.handler(tool_name, t_opening)
        def tool_opening(e, s, args):
            tool.call_handler(tool_name, t_closed, e, s, args)

        @tool.handler(tool_name, t_open)
        def tool_opening(e, s, args):
            tool.call_handler(tool_name, t_closed, e, s, args)

        @tool.handler(tool_name, t_closing)
        def tool_opening(e, s, args):
            tool.call_handler(tool_name, t_closed, e, s, args)
