from outpost_server.core.engine import EngineProxy
from outpost_server.core.extra import ExtraHashProxy

def schedule(eng, when, code):
    return eng.schedule_timer(when, code)

def cancel(eng, cookie):
    return eng.cancel_timer(cookie)

def callback(eng, userdata):
    userdata(EngineProxy(eng))

def init(hooks):
    hooks.timer_fired(callback)


class ObjectTimer:
    __slots__ = ('obj_id', 'when', 'code', 'key', 'cookie')

    def __init__(self, obj_id, when, code, key):
        self.obj_id = obj_id
        self.when = when
        self.code = code
        self.key = key
        self.cookie = None

    def __call__(self, eng):
        obj = eng.get_object(self.obj_id)
        if obj is None:
            return

        # Check that the object's pending timer is actually this timer.  This
        # prevents a spurious wakeup when the timer fires after the object has
        # been unloaded and its ID has been reused for a different object.
        dct = obj.extra().get(self.key)
        if not isinstance(dct, ExtraHashProxy):
            return
        dct = dct.copy()
        if 'when' not in dct or dct['when'] != self.when:
            return
        if 'cookie' not in dct or dct['cookie'] != self.cookie:
            return

        del obj.extra()[self.key]
        self.code(obj)

def schedule_obj(obj, when, code, key='pending_timer'):
    if key in obj.extra():
        cancel_obj(obj, key)

    t = ObjectTimer(obj.id, when, code, key)
    t.cookie = schedule(obj.engine, when, t)

    obj.extra()[key] = {
            'when': when,
            'cookie': t.cookie,
            }

def cancel_obj(obj, key='pending_timer'):
    e = obj.extra()
    if key not in e:
        return

    cookie = e[key]['cookie']
    del e[key]

    cancel(obj.engine, cookie)
