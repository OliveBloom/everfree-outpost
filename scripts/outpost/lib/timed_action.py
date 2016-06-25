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

NO_ICON = DATA.animation_id('activity//none')
def timed_action(delay, icon, check=None):
    icon = DATA.animation_id(icon)

    def decorate(f):

        @functools.wraps(f)
        def g(e, *args):
            if check is not None and not check(e, *args):
                return

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
                    e_.set_activity(Work(anim, NO_ICON))
                    e_.set_activity(Walk())

            cookie = timer.schedule(e.engine, e.engine.now() + delay, callback)
            e.extra()['work_timer'] = cookie

        return g

    return decorate
