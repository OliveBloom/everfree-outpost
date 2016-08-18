import functools

from outpost_server.core import timer
from outpost_server.core.data import DATA
from outpost_server.core.engine import ObjectProxy, Walk, Work
from outpost_server.outpost.lib.util import facing_to_dir

def refresh(eng, x):
    if isinstance(x, ObjectProxy):
        return x.refresh(eng)
    else:
        return x

def mk_fixed_delay(delay):
    def fixed_delay(*args, **kwargs):
        return delay
    return fixed_delay

class TimedAction:
    def __init__(self, icon, finish, delay=None, check=None):
        self.icon = icon
        if check is not None:
            self.start = check
        elif delay is not None:
            self.start = mk_fixed_delay(delay)
        else:
            self.start = None
        self.finish = finish
        self.icon = DATA.animation_id(icon)

    def check(self, f):
        self.start = f
        return f

    def __call__(self, e, *args):
        # Check how long to delay
        delay = self.start(e, *args)
        if delay is None:
            return

        run(self.finish, self.icon, delay, e, *args)


def action(icon, delay=None, check=None):
    def decorate(f):
        d = TimedAction(icon, f, delay, check)
        return functools.wraps(f)(d)
    return decorate

def run(f, icon, delay, e, *args):
    anim = DATA.animation_id('pony//stand-%d' % facing_to_dir(e.facing()))
    e.set_activity(Work(anim, icon))

    cookie = None

    def callback(eng):
        nonlocal cookie

        e_ = e.refresh(eng)
        args_ = tuple(refresh(eng, x) for x in args)

        # Make sure it's the same entity
        # TODO: double-check this logic - it relies on cookies not
        # being reused until the timer has been processed
        if e_.extra().get('work_timer') != cookie:
            raise ValueError('entity was replaced before timer fired')
        del e_.extra()['work_timer']

        try:
            f(e_, *args_)
        finally:
            e_.set_activity(Walk())

    cookie = timer.schedule(e.engine, e.engine.now() + delay, callback)
    e.extra()['work_timer'] = cookie


def check_structure_ward(delay):
    def check_structure_ward_impl(e, s, *args):
        ward.check(e, s.pos())
        return delay
    return check_structure_ward_impl
