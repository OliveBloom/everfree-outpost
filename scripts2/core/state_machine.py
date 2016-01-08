from outpost_server.core import timer
from outpost_server.core.data import DATA
from outpost_server.core.engine import StructureProxy

class StateMachine:
    def __init__(self, obj, key='sm'):
        object.__setattr__(self, 'obj', obj)
        object.__setattr__(self, '_key', key)
        if key not in obj.extra():
            obj.extra()[key] = {}
            self.init()

    def __getattr__(self, k):
        try:
            return self.obj.extra()[self._key][k]
        except KeyError as e:
            raise AttributeError(str(e))

    def __setattr__(self, k, v):
        if k in self.__dict__:
            object.__setattr__(self, k, v)
        else:
            self.obj.extra()[self._key][k] = v

    def __delattr__(self, k):
        del self.obj.extra()[self._key][k]

    def _raw_get(self, k, default=None):
        try:
            return object.__getattribute__(self, k)
        except AttributeError:
            return default

    def init(self):
        self.state = type(self).START_STATE

    def process(self, msg):
        self.transition(msg)

    def transition(self, new_state):
        old_state = self.state

        f_leave = self._raw_get('leave_' + old_state)
        if f_leave is not None:
            f_leave()

        self.state = new_state

        f_enter = self._raw_get('enter_' + new_state)
        if f_enter is not None:
            f_enter()

    def schedule(self, delay, msg):
        self.cancel()

        eng = self.obj.engine
        when = eng.now() + delay
        id = self.obj.id
        key = self._key
        cookie = timer.schedule(eng, when, lambda eng: callback(eng, id, key, when))
        self.timer = {
                'when': when,
                'cookie': cookie,
                'msg': msg,
                }

    def cancel(self):
        if hasattr(self, 'timer'):
            timer.cancel(self.obj.engine, self.timer['cookie'])
            del self.timer

_TEMPLATE_SM = {}

def structure(template):
    template = DATA.template(template)
    def f(cls):
        assert template not in _TEMPLATE_SM, \
                'duplicate registration for %s' % template
        _TEMPLATE_SM[template] = cls
        return cls
    return f

def get(obj, key='sm'):
    cls = state_machine_class(obj)
    return cls(obj, key)

def state_machine_class(obj):
    if isinstance(obj, StructureProxy):
        return _TEMPLATE_SM[obj.template()]

def get_state_machine_class(obj):
    if isinstance(obj, StructureProxy):
        return _TEMPLATE_SM.get(obj.template())

def callback(eng, id, key, when):
    obj = eng.get_object(id)
    cls = get_state_machine_class(obj)
    if cls is None:
        return

    timer = obj.extra().get(key, {}).get('timer')
    if timer is None:
        return
    if when != timer['when']:
        raise ValueError('mismatched when (old timer fired after being cancelled)')
    msg = timer['msg']
    del obj.extra()[key]['timer']

    cls(obj, key).process(msg)
