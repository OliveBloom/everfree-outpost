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
